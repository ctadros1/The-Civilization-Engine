# ADR-0019: Exchange between settlements: reach by reports, trades where they settle

Status: Accepted
Date: 2026-10-09
Milestone: M5b

## Context

Each settlement has one market (ADR-0006 §4). A buyer searches only its own settlement's sellers,
within the trade activity's half-hour walk, and pays at the seller's door; asks move toward the
seller's own cost in hours of its own work. Since M5a there are several settlements, people know
others only by contact (ADR-0018 §4), and they visit, marry and move between them. Nothing yet
lets a household buy from a neighbour, so two settlements' prices can only co-move under their
shared weather.

M5b (plan §7) asks for goods carried between settlements (buyers fetching for themselves,
households fetching to resell, then merchant firms), price reports as households' dated beliefs,
and a price convergence row, with a demo in which two settlements trade and their prices close
to a band. How a household learns of and reaches another market, where goods change hands and how
trades are counted is expensive to reverse: saves hold it, the convergence row reads it, and M6's
towns and M7's tariffs and credit build on it.

Reports read: 08-05 (markets and prices), 08-12 (trade and merchants), 08-16 (provisioning),
08-06 (money), 09-16 (information flow) and 16-01 (measuring outputs), digested in the M5 trade
brief (`docs/briefs/m5-trade.md`). Their main points:

- **Shipments, local markets and beliefs, not a global price plus a distance surcharge**
  (08-12 §5.3; 08-05 opening). Physical truth resolves moves and sales; beliefs choose them.
- **Merchants know dated observations, not today's prices elsewhere,** and must estimate depth as
  well as price, or every one of them answers the same shortage and gluts it (08-12 §1.6; 08-05
  §1.4).
- **Gaps close to a no-arbitrage band, not to zero**: a gap smaller than the cost of carrying a
  good persists without trade (08-05 §1.7, its 100/130/25 example), and adjustment comes through
  purchases and arrivals, never a rule that synchronizes towns (08-05 §5 D).
- **A seller's reference price incorporates procurement** and carrying costs, and production cost
  is not an absolute floor (08-05 §1.5).
- **Tradability emerges from the goods,** from value per kilogram, perishability and scarcity
  where they are wanted, never from flags (08-12 §1.2).
- **Money is acceptance, kept per market,** never a global flag (08-06 §1.3).
- **Price correlation is not integration:** shared weather and seasons move disconnected towns
  together (08-12 §4; 08-05 §7), so convergence needs a counterfactual.

## Decision

### 1. A household buys elsewhere only by report

- A household reads its own settlement's offers as they stand, as today. Another settlement's it
  knows only by **price reports**: per household, per market, good and payment good (a seller
  takes several goods for one, and a buyer goes by terms in what it holds), the latest terms it
  holds (price a unit, the seller and its ask in hours, the units offered), the day they were
  seen, and how (seen by a member, or told by whom). Of two as new, one seen beats one told.
- Reports start with a member who was there: one who buys at a seller's door sees that seller's
  offers; one who keeps company at another settlement's hearth hears the offers of the
  households of those they keep company with. They pass between companions at the hearth at a
  keyed chance, as routine news does (09-16 §2.2's 0.05–0.25), and a member's report is the
  household's at once. A report's weight halves with its age, by a half-life that is a tuning
  value.
- Reports are kept per household, like the places it knows (ADR-0018 §4), not per person: some
  5 KB a household that holds them (slice AP measured about 114 reports a household among three
  settlements), so several megabytes at 8,000 people, and a household chooses its purchases
  together.
- No household reads another market's truth, and nothing names one settlement's price in
  another's rules.

### 2. A fetch is a trip to a seller's door

- Buying from another settlement is today's purchase choice, the offer that saves the household
  the most of its own hours with the walk there and back counted, extended to the reported offers
  of settlements it knows within a day's walk. A content activity, `fetch`, carries it, with its
  own walk limit and daylight rule; `trade` within a settlement is unchanged.
- At the door the seller's terms as they stand decide. A stale report can fail (sold out, the
  terms changed, nothing the buyer holds is accepted), and the failure is recorded with its
  reason (or that the household no longer needed what it went for); the buyer's report is
  corrected by what it saw. What the seller no longer offers is held as none left, dated, so that
  older word of it is not believed again. One of a household goes at a time (slice AP).

### 3. Goods change hands at the seller's door

- As within a settlement, the ledger moves the goods to the buyer's household and the payment to
  the seller at the moment of the trade (ADR-0006 §3). The carrier walks home; nothing is held in
  transit in M5b.
- This is honest while trips are under a day and nothing on the way can take, spoil or delay a
  load (no robbery, war, tolls or carts yet; goods that spoil in days do not travel). Goods stay
  owned and counted throughout.

### 4. Trades are tallied where they settle

- The market stays keyed by the seller's settlement. Each trade also records the buyer's
  settlement, so a market serving several settlements is measured (the share bought by outside
  households and how far they came), never declared.
- Acceptance stays per market: a buyer from elsewhere pays in what the seller accepts, and the
  payment counts in the seller's market. A shared money or a third good settling between
  settlements may emerge; neither is set.

### 5. An ask may anchor on replacement

- A household's anchor for a good it offers becomes the lesser of its own cost and its
  replacement cost from the best report it holds of another settlement's market: the reported
  price, valued at its own cost of the payment good, plus the carry hours a unit (the walk there
  and back over a load) (08-05 §1.5).
- Only reports of other markets enter. Neighbours' offers within one's settlement do not, so a
  world of one settlement keeps its asks exactly as tuned.
- As built (slice AQ): each report is believed by its weight, what is not believed falling back
  on the household's own cost; the walk is estimated from the distance at its pace off the
  trails (a belief, and the same after a save and load); a seller beyond the fetch walk does not
  count, and content with no `fetch` activity has no replacement at all.

### 6. Fetching to resell is making to sell

- At its weekly review a household weighs, per good and report, the margin of one trip in its
  own hours: the units times the difference between what it expects to get at home and the
  reported price, less the walk and the trading. The units are the least of a load, the units
  reported and the units it expects to sell at home over a review, from its market's sales and
  unmet demand (08-12 §1.6's depth). The trip then competes in the activity scorer, as making for
  sale does; the goods are offered at its door on its usual terms.
- A household that fetches repeatedly may run it through a firm, by the existing firm rules
  (ADR-0006 §5).

### 7. Convergence is recorded as it happens

- Monthly, per pair of settlements lived in: per good offered in both, the median asks in hours
  and the realised prices; and the units and kilograms carried, the trips and their walking
  hours. Saved, because tallies fade and offers change; the dashboard's price convergence row and
  the demo read it.

### 8. The demo's twin is a harness, not a rule

- A switch, set in memory by a host command or a test and never in content or a save, stops
  purchases across settlements: a trip to buy elsewhere is left out of every choice. A world
  lives identically with it until the first choice that trip would have been part of (not quite
  the first purchase it stops: leaving an option out changes the draw among the rest), so a
  save lived twice gives the comparison the shared weather requires (08-12 §4).

## Consequences

- Easier: two settlements' prices can close by trade alone, to a band set by what carrying costs,
  and every step is explainable from saved reports, trips and trades.
- Easier: M6's towns and M7's credit and tariffs find a market already serving outsiders, its
  buyers and their settlements on record.
- Harder: a purchase may now walk hours, so route searches to other settlements must reuse the
  settlement fields visits use; Accelerated mode's cached household view holds "the purchase"
  (ADR-0011 §4), so Gate B must cover purchases between settlements.
- Forbidden: a global or regional price; buying from a settlement one has no report of; a
  convergence or synchronization rule; flags that make a good local or tradable; an export buyer
  or source without stocks.

## Alternatives considered

- **A global price plus a distance surcharge:** the reports reject it (08-12 §5.3), and it would
  close every gap at once without anyone carrying anything.
- **Reading other markets' offers live:** simpler, but every household would answer the same gap
  at once and glut it (08-12 §1.6), and information would move without people.
- **Reports per person** (as ADR-0016 holds hearing): truer, but a household buys together and the
  memory doubles; revisit if households split their purchasing.
- **Goods in transit as a holder now:** needed once trips last days, carry loads on carts or can
  be robbed; at under a day on foot it changes nothing a person could notice.
- **The replacement anchor for one's own neighbours too:** it would narrow the house-to-house
  spread inside every settlement and change every world of one settlement, which the dashboard
  has graded since M3c.
- **A periodic market day as the venue** (08-05 §1.1): a policy a polity might adopt; it belongs
  with M6's towns unless trade fails without it.

## Revisit when

- Trips last overnight, loads travel by cart, boat or animal, or anything on a route can take
  goods (a goods-in-transit holder).
- Credit, agents or deferred settlement arrive (M7), when trust and default matter.
- Metal content makes minting possible (coins, cohorts and per-polity currency; ADR-0006's own
  trigger).
- Households split their purchasing among members (reports per person).
