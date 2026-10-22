# M10 design brief: receipts, notes and fiat money

**Scope.** Plan §7's M10 ("Industrial content") ends with "fiat money". Plan §5.3 lists money as discovered nodes, of which the last three are "Banking and bills of exchange", "Backed paper" and "Fiat (post-v1)"; plan §9 records "Money: discovered (barter → commodity → coin → credit → paper → fiat); per-polity currencies". Steam and factories, rail and gas, and health and education have their own M10 briefs; this one stays on money and names where it touches theirs. It covers:

- what M7d built of money, and what it planned but did not build;
- why a medium nobody eats or works with is never accepted today, and how acceptance can come from onward exchange and from dues;
- custody and receipts, receipts that pass from hand to hand, and notes a keeper issues beyond what it holds;
- a polity's notes: issue, spending, dues payable in them, legal tender, and convertibility and its suspension;
- over-issue and inflation through posted prices and holders' beliefs, never an index;
- counterfeiting;
- what a money of account changes in prices, credit, wages and taxes;
- what each new behaviour costs, and how it is applied in a household-resolved settlement.

Reports read in full: **08-06** (money history), **08-07** (credit and banking), **08-13** (taxation), **08-20** (economic crises). Skimmed: **08-05** (markets and prices) and **08-04** (agent-based macroeconomics). No report covers counterfeiting; where this brief proposes for it, it says so.

**What exists to build on** (checked in the code):

- **The ledger** (`civ-agents/src/ledger.rs`). Every movement of goods between holders is one operation with a channel; a transfer that cannot be covered does not happen (ADR-0006 §3). There are 26 channels, the latest `Lent` and `Repaid` (M7d slice CA), `Mint` (metal brought to a mint), `Carried`, `GroundFee` and `GroundRent`. Goods are created only by gathering, harvest and recipe outputs (ADR-0006 §3), and flows are counted per good for the conservation check (`person.rs`, `Flows`).
- **Money is inferred per market** (`market.rs`, `Market::money`): the good settling at least `money_share` (0.5) of the remembered payments' worth over at least `money_min_trades` (10) trades, tallies halving every `memory_days` (30) (`content/core/people/early_farmers.toml`). "Nothing in the engine names a money good". The inferred money is the unit of account (ADR-0006 §4). Acceptance stays per market; a buyer from elsewhere pays in what the seller accepts (ADR-0019 §4).
- **Posted terms are hours** (`population/market.rs`, `post_terms`): an ask in the seller's hours (`ask_h`), reviewed weekly by at most 5 % (`review_days` 7, `max_change` 0.05), posted in each payment good as `ask_h` divided by what a unit of that good is worth to the seller. A seller takes a payment good only if it "wants" it at least `accept_want` (0.5) or it is the market's money, and only if it can value it (`costs.get(p).copied().flatten()?`).
- **What a money is worth to its holder** (`population/market.rs`, `pay_worth`, `pay_worth_raw`; M7d slice CD, ADR-0027 §5). For a good with a `[money]` table: the greater of its own cost and what a unit bought in the household's remembered purchases, faded by `memory_days`, less `verify_h`; beyond `money_days` (14) of its spending in it, scaled by desired ÷ held. A coin is worth its metal to one who can assay it (`assayed_worth`, CE4a), and another polity's piece is taken by weight at `foreign_discount` (0.1) to one who cannot (`foreign_worth`, CE4c).
- **The levy floor** (`levy_floor`, CE3b). Under a common store whose template names a `[medium]` (`content/core/policy/common_store.toml`: coin, 2 pieces a kilogram of grain), a unit is worth at least the grain it pays for, but only as far as the levy could take it: beyond `levy_share × grain held × per_kg` units the floor scales by usable ÷ held. A `[medium]` is valid only on a common store (`civ-content/src/policy.rs`, "`medium` is for `does = \"common_store\"`").
- **Coin** (`coin.rs`, `population/mint.rs`; ADR-0027 §6). A `Cohort` per issuer, day, mass and fineness, with its count; a `Purse` per holder (household, workshop, polity's store, faction, payment owed) counting pieces by issue. `Coinage::give` moves the pieces worth least to the payer first; `Coinage::reconcile` puts purses right at midnight, each issue's count conserved. A mint is a law (`PolicyKind::Mint`, `content/core/policy/mint.toml`: 8 g pieces, 3 % fee) naming a keeper who strikes as the duty of an office (`MakeWorth::Office`, `strike_points`); households bring metal at midnight when the pieces are worth more to them than the metal (`bring_to_mints`). A law of pay for common work (`PolicyKind::PayWork`, 8 pieces an hour) pays hours given to the polity's works from its store "as far as the store holds it" (`pay_for_work_given`, at midnight).
- **The polity's store** (`polity.rs`, `Polity::stores`, `storehouse`): the levy (`Channel::Levy`), relief, fines, ground rent and duties go in; relief, store loans (`lend_from_store`, M7d slice CC1) and pay for work come out. A keeper may divert from it for their own household (`Channel::Diverted`, ADR-0026 §4).
- **Claims** (`credit.rs`, `population/credit.rs`; ADR-0027 §1-3): loans between households and from the store, in "the food good it holds most of", kilograms lent and owed, made and due days, a pledge (field or guarantor), standing, witnesses. A claim names its creditor for life: there is no assignment. Obligations from findings (`crime.rs`, `Obligation`, kinds `Fine`, `Debt`, `GroundRent` among them) are counted in kcal.
- **Wages** (`firm.rs`, `WageOffer`; `population/firm.rs`, `plan_wage`): a wage in the owners' hours (`hour_h`) paid "in the settlement's money, or else in what neighbours most accept", reviewed every `wage_review_days` (14) by at most `wage_max_change` (5 %).
- **The household day** (`population/resolution.rs`; ADR-0032 §2): `Trade`, `Ask`, `Hire` and `Make` are shared as blocks (`SHARED`), `Trade`, `Ask` and `Hire` at most once a day (`ONCE`), applied serially at midnight in household id order. `Visit`, `Fetch`, `Carry`, `Band` and the fire behaviours are not taken (`NOT_TAKEN`). Reviews of terms, credit, minting and pay for work already run at midnight (`Population::on_day`).

**What M7d planned and did not build** (plan §9):

- **Custody, keepers lending deposits, and runs (slice CC2) were deferred** on 2026-10-17: in about 40,000 household-days over two seeds no household held more than its roof kept, so "Nobody would deposit, so no keeper would lend and no run could come". ADR-0027 §4's `Custody` channel does not exist in `ledger.rs`. Receipts, the natural first paper, therefore have nothing to be receipts for yet.
- **No lived world has struck coin.** The M7d demo (slice CE5) taught smelting and found nobody smelted: households held 18.7 and 19.3 stone axes and never needed a copper one. "Coin waits on metal, and metal waits on tools wearing out."
- **A money nobody uses is never accepted at a door.** `holding_of` gives a good a `keep` only as food, a tool or a tool's makings, fuel, or building material while unhoused ("a money is not built with"), and `want` is zero where `keep` is zero. Coin therefore has `want` 0, and `post_terms` takes it only once it is already the market's money, which needs trades in it first. Coin can circulate store to household (pay for work) and back (levy in coin), but not between neighbours. Plan §9 lists the gap: "Not yet built: acceptance by those with no use for metal before it is their market's money", and for CE3b, "a household seeking coin to pay its levy". A note has no use and no own cost at all, so without a fix no note would ever change hands.
- **No writing or record-keeping technique** is in the core content (`content/core/technique/`). A note needs a durable, authenticated record (08-06 §5.5).

## Where the reports agree

1. **Five properties, not one Boolean.** Asset backing, convertibility, the issuer's liquidity, tax receivability and acceptance are separate questions (08-06 §1.9). A warehouse receipt, a redeemable note, a transferable deposit and an inconvertible state currency are distinct instruments; "A solvent issuer may be temporarily illiquid; an insolvent issuer may keep paying for a while" (08-06 §1.9).
2. **A claim becomes money only when third parties accept its assignment** (08-06 §1.2; 08-07 §1.7, §5.5's "Assignment and negotiability"). Transferring a claim changes its holder; it never copies it (08-07 §5.2).
3. **Dues create demand, decrees do not create acceptance.** Accepting a medium for enforceable dues makes it more useful; requiring it for private debts and requiring merchants to sell at a stipulated price are different policies, answered by repricing, hidden premiums or withdrawal from trade (08-06 §1.4). A household owing a tax in a medium seeks that medium (08-06 §1.4); wealth is not liquidity (08-13 §1.5).
4. **Paper is neither modern nor doomed.** Sichuan's private claims became official jiaozi in 1023 with reserves and conversion; Yuan notes ran from full convertibility to an inconvertible standard from 1310, with moderate inflation for nearly half a century and excessive issue tied to military pressure (08-06 §3; 08-20 §1.3). A state can lose an issuing capability (08-06 §3, Ming). "Paper becomes fiat, therefore immediately collapses" is a poor rule (08-06 §3).
5. **A managed fiat system needs "Durable fiscal and monetary institutions plus continued demand for the unit—not merely a printing press"** (08-06 §5.5). Stabilization needs a credible change in financing; deleting zeros fails (08-06 §1.10, §4).
6. **Inflation runs through budgets and decisions** (08-06 §1.10): new spending at particular suppliers, their price decisions, wages and contracts on their own schedules, holders' desired balances, institutions' changed real resources. Exchange rates move first, goods and wages slowly (France 1724). Hyperinflation is a loop (fiscal distress → monetary financing → falling real demand for the currency → faster spending and substitution → larger needs), not a status; Cagan's 50 % a month is a classification, not a switch (08-06 §1.10, §2.2).
7. **Accounting is stricter than behaviour.** Every financial asset has its issuer's liability; backing is not double spent; collecting money transfers it, retiring a liability is another operation; no reserve multiplier (08-06 §5.3; 08-07 §1.6). Constant money is not an invariant, but creation and retirement need explicit entries; repricing is not income (08-04 §4.1).
8. **Runs and failures are balance sheets and expectations.** A run comes when demands exceed reserves, near receipts, emergency funding and sale proceeds; fear of others withdrawing can bring down a viable issuer (08-07 §1.9). Insolvency and illiquidity differ (08-20 §1.4). Payment order in a run is "an institutional rule—not an accidental consequence of memory order" (08-07 §5.6).
9. **Fixed nominal dues and debts are redistributions.** Inflation lightens money debts and leaves grain debts alone (08-20 §1.3); a fixed obligation weighs more in a bad year than a proportional one (08-13 §1.5; 08-20 §1.1). Show nominal prices, prices in a stable standard and real wages apart, or a monetary crisis and a famine look alike (08-20 §1.3, §4).

**Where they differ, or differ from the plan:**

- **The ladder.** Plan §5.3 and §9 order the nodes; 08-06's executive recommendation and §3 reject a universal sequence and allow reversal. I follow the reports: each instrument is a choice that can be made, lost or never made.
- **Fiat as post-v1.** Plan §5.3 marks fiat "post-v1" and §7 puts it in M10. They agree: v1 closed with M8.
- **Paper or tally.** 08-06 §5.5 asks for "Durable documents, authentication, issuance administration". It names no medium; a notched, split tally kept by both parties would meet it without writing. Which record M10's content offers is open (question 1).
- **Exchange rates.** Plan §5.3: "from trade balance and trust". For notes there is no metal parity (08-06 §1.8), so a rate between two polities' notes can only come from what each buys and what each's dues take: the plan's "trust" read as holders' valuations.
- **Build order.** 08-20 §5.4 and 08-06 §5.6's "Recommended v1 boundary" ask that physical stocks and elementary obligations be stable before deposit creation and clearing. TCE's coin has never lived; notes on top of an unlived coin would be built on nothing measured.

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| Custody | Deposits per depositor at a keeper, in a good; the goods stay the depositor's | A household holding more than its roof keeps; a keeper's fee |
| A receipt or note issue | Issuer (household or polity), the good and amount a piece is redeemable in (or none), where and when redeemable, its mark, pieces outstanding | A keeper's or a law's choice to issue |
| Notes held | Purse counts by issue, as coin's purses | Payments through the ledger |
| A redemption queue | Pieces presented and not yet paid, in the issuer's order | Holders' choices |
| A counterfeit | Purse counts of a hidden issue imitating a real one, its maker | A household's choice to make and pass them |
| Dues in a medium | Per law: the medium, its conversion schedule | A law, or its amendment |

**Derived, not saved:** "bank", "banknote", "fiat" and "legal tender" labels; an issuer's coverage (what it holds of the redemption good over pieces outstanding); exchange rates; any price index; inflation; the dashboard's measures.

**Size (my arithmetic).** At M9's 100k people in ten settlements, about 20,000 households. An issue is keyed by issuer and terms; with a few keepers and polities issuing, purses gain a few `(issue, count)` pairs a household. Coin's issues are keyed by day too (`Coinage::strike` makes "an issue of their own, or the day's issue already struck to the same terms"), so a mint striking daily for fifty years would leave 18,000 issues and long purses; notes should not copy that.

## 2. Acceptance for onward exchange and for dues (the blocker)

**Reports.** A holder may accept a good "for either direct use or expected onward exchange", valued separately, less carrying, verification and theft (08-06 §1.3); keep "a local acceptance estimate, updated from successful and refused payments" (08-06 §1.3), 10–100 observations a half-life (08-06 §2.3). Dues payable in a medium make holders seek it (08-06 §1.4). Desired balances are 7–30 days of spending (08-06 §2.3).

**My proposal.**

- **A money's want comes from its uses as money.** For a good with a `[money]` table, `holding_of` gives a `keep` of the household's desired balance (`money_days` of its spending in it, as `pay_worth_raw` already reckons it) plus what it expects to owe in it before its next review (the levy at its share, ground rent, a fine or a debt due, at the law's conversion). `want` then follows by the existing rule. A household that owes in a medium and holds none wants it, and a seller takes it in payment.
- **A household seeks the medium its dues ask.** Short of what it owes, it sells for it: its sale offers take the medium (as above), and its `Trade` choice may favour buyers paying in it. This is 08-13 §1.5's liquidity: wealth in grain, dues in notes, sold at harvest.
- **An acceptance belief per household, per money.** Successful and refused payments in it, fading with `memory_days`, tell a holder whether its neighbours take it; a refusal is recorded when a buyer offers a money a seller does not post, beside the demand nobody met that the market already records (`Market::record_unmet`). It lowers the onward value below what a unit bought.
- **Build it for coin first.** The fix is the same for coin and notes, and coin has a lived path once metal is used. A world with no money good gives the build-before's digests.

## 3. Custody and receipts

**Reports.** Safekeeping holds identifiable property; pure custody covers 100 % (08-07 §1.6, §2.3). "Metal held in a warehouse is not simultaneously available in a household purse because the household owns a receipt" (08-06 §5.3).

**My proposal.**

- **Measure demand again before building**, where M8 and M9 changed the answer CC2 found: towns of thousands (M8), the common store under a storehouse (`yourselves_raise_a_storehouse`, M8d slice DC), and harvests that outrun roofs. If no household holds more than its roof keeps, custody stays deferred and receipts go with it.
- **Custody as ADR-0027 §4 decided:** a household with room keeps another's grain for a fee in grain, on the appended `Custody` channel; the grain is counted as the depositor's, the keeper's store holding it apart. A polity's storehouse may take deposits under a law (a policy kind appended, *keep deposits*, its fee the template's).
- **A receipt is a note of one issue per keeper and good**, made when grain is deposited (a recipe of the record's makings and a minute's work), handed to the depositor, and redeemable at the keeper's door for its grain. Until receipts may pass (§4), a receipt is the depositor's alone and is only a record: it never pays.

## 4. Notes that pass, and keepers who issue beyond what they hold

**Reports.** Assignment needs record authentication, assignment rules and credible settlement (08-06 §5.5). Joint liability let French bills support distant trade without banks (08-07 §1.7). A loan creates a deposit only where the lender's liabilities are accepted for payment (08-07 §1.6). Stockholms Banco's notes promised coin; expanding issue and redemption demands brought it down (08-06 §1.9). Liquidity targets: 10–100 % of redeemable liabilities (08-06 §2.3); a fractional intermediary's prudence prior 10–40 % (08-07 §2.3); "Reserves are not capital" (08-07 §1.6).

**My proposal.**

- **A law makes notes pass:** an appended policy kind, *bearer heard*, under which the gathering or a forum hears a holder's case against an issuer who refused to redeem, as *debts heard* does for loans (ADR-0027 §3). Without it a note can still change hands where people choose to take it, at their own risk, valued by their belief of the issuer; with it, refusal has a remedy. Nothing makes a note pass by fiat of the engine.
- **A note is a good with a `[money]` table and a `[note]` table** (its issuer and redemption terms per issue), held in stores and purses as coin is, moved through the ledger, and valued by `pay_worth`: the greater of what it bought, its dues floor, and, while convertible, its redemption good's worth to the holder times the holder's belief that the issuer pays (from its `Word` evidence of the issuer and what it has heard of refusals), less the walk to the issuer's door.
- **Issuing beyond backing is the keeper's choice.** A keeper whose receipts pass may lend in notes: the loan is a claim on the borrower (the existing book, the good owed being the note) and new pieces of its issue. It weighs it by 08-07 §1.3's rule as it weighs a loan in grain, and keeps a coverage it chooses from a prior (open question 7). Whether a keeper may is a law's question, as ADR-0027 §4 left lending deposits.
- **Illustration (my arithmetic).** A keeper holds 1,000 kg of deposited grain against 1,000 receipts and lends 600 more in notes: 1,600 redeemable against 1,000 kg, 62.5 % coverage, inside 08-06 §2.3's range and above 08-07 §2.3's 10–40 %. A run needs presentations above 1,000 kg plus the loans' repayments due and what it can sell (08-07 §1.9).
- **A run is holders' choices.** A holder who hears that the keeper lent beyond its grain, that a debtor of the keeper defaulted, or that others were refused, weighs presenting; seeing others present weighs more. Presentations queue; the keeper pays in the order its law names, else a seeded order, never household id order (08-07 §5.6). A keeper who cannot pay suspends, and its notes' worth falls by holders' beliefs.

## 5. A polity's notes, dues, legal tender and convertibility

**Reports.** Sichuan's official arrangement: a 3 % conversion charge, 360,000 guan of reserve against an issue ceiling of 1.25 million (about 29 %), issues meant to last three years (08-06 §2.1). Treasuries borrow against taxes; borrowing and monetary financing are financing, not revenue (08-13 §1.7, the cash identity with M; 08-07 §1.8). Store an obligation's acceptable media and conversion explicitly (08-06 §1.4).

**My proposal.** Each a law the gathering may pass, answer an issue, carry its sponsor's forecast and be repealed, with nothing weighing toward it:

- ***Issue notes*** (policy kind appended): the store's keeper or a named office makes notes of the polity's issue (a recipe and an office's points, as striking is), up to the law's ceiling of pieces outstanding, redeemable at the store in a good at the law's rate, or not at all. Its terms are the template's: ceiling, redemption good and rate, conversion fee.
- ***Spending*** is the existing pay for common work, which already names a money; buying grain for relief at posted prices, and paying offices, are open (question 8). A store issues only to spend: a note made and kept in the store is no liability.
- ***Dues in notes***: the `[medium]` table generalised beyond the common store, so a fine, ground rent and a duty may name a medium and its conversion (obligations keep kcal for findings; a medium names what discharges it, 08-06 §1.4). The levy floor (`levy_floor`) then applies to each.
- ***Legal tender*** (appended): a creditor of a claim in another good must take the polity's notes at the law's rate. A creditor who refuses is heard under *debts heard* as the debtor's case. 08-06 §1.4's third policy, a stipulated price at sellers' doors, is deferred: sellers answer by hidden premiums and withdrawal, which needs bargaining TCE does not have.
- ***Suspend redemption***: the store stops paying notes in their good. Holders' beliefs answer: the redemption term leaves `pay_worth`, and what the notes buy and the dues floor remain. A polity's notes that nobody may redeem and its dues still take are what the observer labels fiat. Whether they keep their worth is the levy's and the market's answer, not the label's.
- **Illustration (my arithmetic, from the code's floor).** Under the core's levy medium (2 units a kg) and a tenth's levy, a household holding 1,000 kg of grain gets the floor's full support on up to 200 units; holding 400, each is worth half the floor. Notes issued beyond what dues absorb lose the floor's support, unit by unit, already, in `levy_floor`.

## 6. Over-issue and prices

**What the code implies.** Asks are anchored in hours and move at most 5 % a week; a price in a money is the ask in hours over the money's worth to the seller. More notes reach prices only by lowering their worth: through holders above their desired balance (`money_days`), through what notes last bought, and through the dues floor's scale. Wages are hours converted to the pay good every 14 days, so they lag (08-06 §1.10). Claims in grain do not move; claims in notes lose in hours (08-20 §1.3).

**My proposal.**

- **No index, no coefficient** (08-06 §6). The first sellers the store pays hold more notes and ask more of them; their purchases teach their neighbours that a note buys less.
- **The loop is the polity's own choices.** A store that issues to pay for works when its levy in kind falls short, whose levy in notes buys less each year at a fixed conversion, and whose gathering raises the ceiling, lives 08-06 §1.10's loop. A conversion fixed in notes is a fixed nominal tax: its real yield falls as notes lose worth (08-13 §4.3: "Revenue categories can mislead"). Amending the conversion is a law.
- **Stabilization is a law too:** a new issue at new terms, or redemption restored with the store's grain; whether people believe it is their `Word` evidence of the polity.

## 7. Counterfeiting

No report covers it. 08-06 §5.5 asks for authentication, §1.3 counts verification costs, and 08-13 §4.1's evasion heuristic (gain less perceived chance times penalty, less concealment and moral cost) fits any hidden wrong. These are design priors.

- **A mark costs its maker hours.** An issue's mark (the template's) takes minutes a piece to make well; imitating it takes a member who knows the issue's making technique and more minutes a piece.
- **Making and passing are a taking's kind of choice** (ADR-0015 §2): a household weighs gain against the chance it believes of being found and the law's bundle. The counterfeit pieces are a hidden issue in purses, counted by holders as the real one, as an unassaying holder counts coin (CE4a).
- **Telling them apart is a skill.** Those who know the making technique tell at payment and refuse; the issuer always tells at redemption and keeps them. A refused or kept piece is an incident with its passer known or not, heard under an appended *against counterfeiting* law (an `AgainstTaking`-shaped kind).
- **Acceptance answers.** Refusals lower holders' acceptance beliefs (§2); an issuer may answer with a costlier mark on a new issue.

## 8. What a money of account changes

- **Prices.** Nothing new: asks stay in hours, posted in each payment good; the market's inferred money becomes a note issue only if most remembered payments settle in it (ADR-0006 §4). The observer shows prices in notes and in hours side by side (08-20 §1.3).
- **Credit.** A claim may be lent and owed in a note issue (the book's `good` already names any good). A keeper's loans in notes are claims in notes. Inflation moves their burden in hours; grain claims stay. Legal tender (§5) decides whether a grain or coin creditor must take notes.
- **Wages.** `plan_wage` pays in the market's money when the workshop holds it, so notes become wages where they are the money, reviewed every 14 days: real wages fall while asks rise weekly (08-06 §1.10; 08-20 §1.2's medieval food prices outrunning wages).
- **Taxes.** A levy in kind follows the harvest; a levy in notes at a fixed conversion is fixed nominally and falls in real terms with inflation. Obligations in kcal stay in kcal; only their medium of discharge changes. The polity's store keeps notes it collects ("Collecting coins transfers ownership to the treasury. It does not physically destroy them", 08-06 §5.3) and may spend them again.

## 9. The ledger's invariants

- **No money without a recorded act.** A note piece comes into being only by an issue act (the recipe's output, its issue's outstanding count raised in the same operation), and a counterfeit only by its maker's recipe, recorded as its hidden issue. Nothing creates pieces by revaluation, reconciliation or a law's passing.
- **Every piece outstanding is its issuer's liability.** Per issue, pieces in purses other than the issuer's equal pieces made less pieces retired; a piece in the issuer's own store is not outstanding. Redemption moves pieces to the issuer and the good to the holder on an appended `Redeemed` channel; retiring is a separate, recorded destruction (08-06 §5.3).
- **Backing is never counted twice.** Grain in custody stays the depositor's on `Custody`; its receipt is a claim, not grain (08-06 §5.3).
- **Counterfeits are never anyone's liability**, and are destroyed only by a recorded act when kept at redemption.
- **The conservation check extends** to per-issue counts, as `Coinage::reconcile` keeps coin's, and a debug check that no issue's outstanding count goes negative.

## 10. Household-resolved settlements and cost

ADR-0032's Consequences: "Every behaviour added after M9 is written once and must name how its effect is applied as a block, or be listed as not taken". Its amendment of 2026-10-21 (slice DN) left M9 at about 2.2 ms a person-day at 100k, 140 times short of its gate; money must not add a scan per decision.

- **Depositing, withdrawing and presenting notes** are a `Trade`-shaped block, at most once a day (`ONCE`), applied at midnight with its target's stores as applied. Presentations in a household-resolved settlement join the issuer's queue at midnight and are paid in the issuer's order or a seeded one, never in the household id order blocks are applied in.
- **Issuing and counterfeiting** are `Make` blocks through `make_work`, as striking coin is (an office's points or a household's gain).
- **Passing notes** is ordinary `settle_trade`; refusals are recorded by the trade block.
- **Valuation must be cached.** `pay_worth_raw` scans every market's `recent` trades (64 a market, `recent_trades`) on each call, and `best_deal` calls `pay_worth` per offer's payment good inside a decision. With 30 settlements that is about 1,900 trades a call; each new money multiplies it. A per-household, per-money table of worth and acceptance, built at midnight in the parallel planning pass, would cost about 20,000 × a few numbers at 100k.
- **Midnight work stays sparse:** redemption queues and issues are per issuer, claims on the maturity queue (08-07 §5.6; 08-06 §5.4), never a scan of every purse.
- **Bench before default:** `bench_village` at ten settlements with and without an issue in force; a world with no money good gives the build-before's digests.

## 11. Guards

- No date, era, technique count, population or wealth opens notes, legal tender or fiat; nothing weighs toward issuing; no inflation coefficient; no index read by behaviour; no hyperinflation status; no forced suspension; no rescue that creates goods or notes (08-20, core recommendation).
- A polity may lose an issuing capability (08-06 §3): a law repealed, a keeper gone, notes refused.
- If nobody issues or takes notes after a time box, nudge a prior (a keeper's coverage, the acceptance belief's start), never an issue, and log a `NUDGE:` (plan §1, rule 3).

## 12. What to defer

| Defer | Why |
|---|---|
| Bills of exchange and clearing between settlements | Correspondents, recourse and net settlement (08-06 §1.8; 08-07 §1.7); visits and fetches are not taken while household-resolved |
| Transferable deposit accounts and cheques | Needs accepted ledgers and settlement (08-07 §5.5) |
| Public debt and bonds | 08-07 §1.8; a polity has no budget yet |
| Price ordinances at sellers' doors | Hidden premiums and withdrawal need bargaining (08-06 §1.4) |
| Money changers as firms | 08-06 §1.8; needs several moneys in quantity |
| A central bank, deposit insurance, emergency lending | 08-07 §1.9; 08-20 §1.6: a credible funder first |
| Redenomination | 08-06 §1.5, §4: a scale change with no real effect unless financing changes |

**Stylized facts, as plausibility checks:** paper can persist for decades before deteriorating (08-06 §4); a monetary change without a harvest change and the reverse look different in hours (08-20 §4); old and new issues coexist (08-06 §4); prices move by goods at different speeds, wages last (08-06 §1.10); a run can fell a solvent keeper, and a keeper can fail without one (08-07 §1.9; 08-20 §1.4); lenders neither all fail nor all grow rich (08-07 §4).

## 13. Suggested slices

1. **Acceptance for onward exchange and for dues**: a money's keep from its desired balance and expected dues; an acceptance belief per household and money from paid and refused payments; a household seeking the medium its dues ask; dues in a medium beyond the common store. Measured with coin in a world where copper is used (the observer's knowledge tool), and a world without money giving the build-before's digests.
2. **Custody, re-measured, then receipts**: the deposit-demand probe in towns and storehouses; if it finds demand, the `Custody` channel, keepers' fees, *keep deposits*, receipts as a record that does not pay.
3. **Notes that pass**: note goods with issues and purses (sharing coin's record, issues keyed by terms), redemption and its queue, *bearer heard*, valuation by redemption belief, the cached worth table.
4. **Keepers issuing beyond backing, and runs**: loans in notes by 08-07 §1.3, coverage chosen from a prior, presentations by belief and by seeing others, the issuer's payment order, suspension.
5. **A polity's notes**: *issue notes* with ceiling and terms, spending through pay for work, dues payable in notes, *legal tender*, *suspend redemption*; the derived fiat label.
6. **Counterfeiting**: marks and their hours, making and passing as a weighed wrong, detection by skill and at redemption, *against counterfeiting*.
7. **The observer and the M10 money demo**: prices in notes and hours, real wages, coverage, redemption queues and refusals on the wealth and market panels, chronicle lines for issues, suspensions and runs; the demo time-boxed, nudges logged.

## 14. Open questions for the designer

1. **The record.** What makes a note: a notched tally of wood (no writing), or paper and writing from the health and education brief's literacy? Without a record technique, notes cannot be made.
2. **Order.** Is slice 1 (acceptance) built even if no world has coin yet, since notes need it more than coin does?
3. **Custody.** If the probe again finds no deposit demand, do notes begin with a polity's issue alone (slices 3 and 5 without 2 and 4)?
4. **Inferred commodity money.** Does slice 1's acceptance belief apply to grain and firewood too (the M7 brief's question 7), changing every world and needing a dashboard rerun?
5. **Unit of account.** Inferred per market, as now, or may a law name a reckoning unit for dues and claims?
6. **Issues.** Keyed by issuer and terms only, and should coin's daily cohorts be merged the same way?
7. **Coverage.** Is a keeper's coverage a content prior, its own learned belief, or a law's term?
8. **Spending.** Beyond pay for common work, which first: buying grain for relief, paying offices, or paying a band?
9. **Legal tender.** In M10, or only dues in notes?
10. **Between polities.** Is another polity's note taken only by what it has bought and what dues abroad accept, with no parity?
11. **Counterfeiting.** Who can tell a false piece: knowers of the making technique, anyone after use, or only the issuer?
12. **Demo.** Does the M10 demo seed metal or a record technique by the observer's knowledge tool, as M7d taught smelting, given that no lived world has made coin?

## 15. What an ADR must decide

Acceptance of a money with no use (its keep, the acceptance belief, seeking a medium for dues); the note's record (good, issues, purses, redemption terms) and whether it shares coin's; the appended channels (`Custody`, `Redeemed`) and the per-issue liability invariant; the policy kinds (*keep deposits*, *bearer heard*, *issue notes*, *legal tender*, *suspend redemption*, *against counterfeiting*) and dues in a medium beyond the common store; a run's payment order; how each new behaviour is applied in a household-resolved settlement and the cached worth table; the guards and the demo's nudges.
