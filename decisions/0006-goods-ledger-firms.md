# ADR-0006: Goods, the ledger, prices and firms

Status: Accepted
Date: 2026-10-04
Milestone: M3a

## Context

M3a gives the village an economy (plan §5.3, §7). Goods are made by recipes, tools wear out,
people gain skills, exchange happens at posted terms with barter first and commodity money
later, and firms keep books and hire labour. That adds world state that saves will keep for
decades and panels will show:

- what each holder has;
- what moved between holders, and why;
- the terms posted and paid;
- every firm's books.

Reports read, through six briefs:

| Area | Reports |
|---|---|
| Economy | 08-02, 08-03, 08-04, 08-05, 08-06, 08-08, 08-10, 08-14, 08-15, 08-21 |
| Technology | 07-04, 07-05, 07-06, 07-07 |
| Other | 03-05, 06-08, 02-04, 16-01 |

Their main points:

- **Accounting is stricter than behaviour.** Agents may misjudge, but the books may not. There is
  no double spending, no duplicated goods, no free production, no income from repricing, and
  nothing vanishes when a firm closes (08-04 §4.1; 08-06 §5.3; 08-08 §5.6).
- **Money is acceptance, not a flag.** People accept a good to use it or to pass it on. Money
  goods stay in ordinary stores. The unit of account is not the medium of exchange (08-06 §1.3).
  An early village does not have to adopt money at all (08-06 §3).
- **Transfers have channels:** household allocation, reciprocal gift, barter and sale. Markets
  sit beside household production and sharing; they do not replace them (08-06 §1.1, §5.2;
  08-21 §4).
- **Firm accounting.**
  - An early enterprise can be an activity account inside a household (08-08 §1.1).
  - Its books need cash, inventories, productive assets, receivables, debt, owner claims, income,
    expenses and due dates (08-08 §5.3).
  - Profit is not cash (08-08 §1.6).
- **Tools are requirements and specific upgrades.** They are not a bonus on work rates that
  already assume them (08-03 §1.2; 08-02 §3.1). Condition goes with use (07-06 §6.4).
- **Measure demand, not sales.** Requests a buyer could pay for, stockouts and unfilled orders
  are recorded apart from sales (08-04 §1.2; 08-05 §6; 08-15 §5.5).
- **Prices must not allocate without goods moving.** Demand that goes unfilled stays unfilled
  (02-04 §1.1, §4.6).

## Decision

### 1. Goods and their units

- A good is counted in its unit: kilograms, or, for a **tool**, the life of a standard tool
  (content `life_h`, hours of use).
  - A store of 2.4 sickles is two sickles and what is left of a third.
  - An hour's use takes 1/`life_h` of a unit.
  - A tool made with more skill adds more than one unit: its quality is its life.
- Stores stay one number per good, kept by content id in the households section (ADR-0003).
  New goods need no schema change.
- Each good says how it is eaten: as it is, cooked over a fire, or not at all. A good that is
  not eaten must go through a recipe first.

### 2. Recipes and skills

- A **recipe** is content:
  - inputs and outputs in their units, per unit made, and per session (firing an oven);
  - labour per unit and per session;
  - the tools it needs and wears;
  - the skill it trains;
  - where it is worked.
  
  Working a recipe is an activity like any other.
- A person's **skill** in a domain is a level from 0 to 1.
  - Practice raises it: s′ = s + (c − s)(1 − e^(−kE)), with k = ln 5 / T80 (06-08 §5.2).
  - It sets work speed and the life of the tools a person makes.
  - Skills are saved per person, sparse, by skill content id.

### 3. Holders and the ledger

- **Holders** of goods are households and firms. ADR-0007 adds settlements, for what a community
  holds.
- **Transfers.** Every movement of goods between holders goes through one ledger operation.
  - The operation moves exact quantities and carries a channel: `gift`, `share` (a shared kill),
    `barter`, `sale`, `wage`, `rent`, `owner` (a firm's owner draws or puts in) or `inherit`.
  - Quantities are reserved before a transfer is committed.
  - A transfer that cannot be covered does not happen, so stocks are never negative.
- **Creation and destruction.**
  - Goods are created only by gathering, harvest and recipe outputs.
  - They are destroyed only by eating, burning, spoilage, wear, building and recipe inputs.
  - A conservation check runs in the tests and the smoke seeds. Over all holders it checks that
    opening stock, plus what was created, minus what was destroyed, equals closing stock.

### 4. Terms, acceptance and the unit of account

- **Posted terms.** A seller posts terms per good it sells: so much of a payment good per unit.
  - It reviews them on its own schedule, by default weekly.
  - A review moves from a cost anchor and from its stock against expected sales (08-04 §1.2;
    08-05 §1.5), within an authored largest change.
- **Paying.**
  - A buyer pays in goods the seller accepts.
  - A seller accepts a good it needs for its own use, or one whose acceptance is high.
  - A buyer offers what it needs least.
- **Acceptance.** Each settlement keeps, per good, the share of payment value settled in that
  good over a trailing window.
- **The unit of account is inferred.** It is the good with the largest acceptance share, once
  that share passes an authored threshold.
  - The observer calls it the settlement's commodity money; nothing in the engine sets it.
  - Until a settlement has one, books keep quantities only.
- **Recorded demand.** Requests a buyer could fund but found no seller for are recorded, and so
  are stockouts, as well as sales.

### 5. Firms

- A **firm** is an enterprise account with stores of its own: its inventory, its tools and
  stations, and its till.
- In M3a its only legal form is the **household workshop**. One household owns it; its members
  work for it unpaid and draw its surplus through the `owner` channel.
- **Record:**
  - permanent id, name, owner and founder;
  - when it was founded, and when it closed and why (completion, voluntary exit, owner's line
    ended, failure);
  - its recipes and the goods it sells, its posted terms, its wage offers and its stores.
- **Books:**
  - a bounded ring of the latest ledger entries that touched the firm;
  - monthly statements, kept for the firm's life: income and costs in the unit of account, and
    stocks at the month's end, valued at its terms or at cost.

### 6. Saves and boundary

- **Saves** gain an `economy` section holding:
  - firms: their records, stores, terms, offers and books;
  - each settlement's acceptance window and price history;
  - recorded demand.
- **Skills** join the people section. Both changes bump `SAVE_SCHEMA_VERSION`.
- **Older saves** load with no firms, no prices and no acceptance history. Their people get
  skills drawn as a founder's are, and their households a founding family's tools.
- **The boundary** gains, all additive:
  - queries for markets (terms, recent trades, price history, demand) and firms (record, stores,
    books);
  - an economy revision in the snapshot.

## Consequences

- Every exchange, wage and rent can be inspected and conserves goods, so a panel can always say
  where something came from.
- Money cannot be scripted. A good becomes money only through what people accept, and a
  settlement may never get one.
- Books cost memory: statements are kept for a firm's whole life, about 1 KB a firm-year.
- Forbidden:
  - changing a holder's stores outside the ledger, apart from the creation and destruction in §3;
  - prices that move goods by themselves;
  - a firm without an owner.

## Alternatives considered

- **Tools as items with their own condition.** This needs more state, a schema change per tool
  and item transfers. The fractional unit gives the same economics with one number: life is
  used, and life is made. Revisit if tools need provenance (who made this sickle) or repair one
  by one.
- **A fixed unit of account** (grain, or days of labour). This scripts the money good.
- **A firm's stock kept in its owner's stores.** The books could not tell the business from the
  household.
- **A market-clearing price per good per day,** a Walrasian auctioneer. Prices would allocate
  without goods moving, and the plan rules out an equilibrium solver (§5.3).

## Revisit when

- Credit arrives (debts as holdings of their own; §5.3, node 4), or coinage (a minted good with
  an issuer).
- Firms get legal forms beyond the household workshop (M4 onward), or employees from other
  settlements (M5).
