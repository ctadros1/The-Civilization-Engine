# M7 design brief: credit, banking, coin and debasement

**Scope.** Plan §7's M7 ("Politics deepens") lists "Credit and banking; coin debasement and inflation". Plan §5.3 makes money "discovered technology/institution nodes" (barter, commodity money, coinage "minted by an authority, with seigniorage and the option to debase", credit and moneylending, banking and bills, backed paper), says "Once polities mint, **each polity has its own currency**. Exchange rates emerge from trade balance and trust", and lists "Saving, spending and lending" among agent decisions. Plan §5.2 gives offices the power to "mint currency" and puts "Currency: minting and debasement" in the policy library. This brief covers, for polities of 100–3,000 people:

- credit between households, and with workshops: loans, interest, due dates, default, pledges and guarantors, and the risk of debt bondage;
- who can lend, and what a bank is at village and town scale;
- commodity money as it stands, and how coin could come to exist;
- debasement as a policy, and how prices answer through posted asks, never through an index the engine sets;
- how per-polity currencies meet in trade between villages;
- crises (defaults, runs) as mechanisms.

Plan §4.3 sets 3–8k people in 2–3 settlements for M5–M7. Taxation, tariffs, courts and guilds have their own M7 briefs; this one names where they touch money.

**What exists to build on** (checked in the code):

- **The ledger** (`civ-agents/src/ledger.rs`). Every movement of goods is one operation with a channel, and a transfer that cannot be covered does not happen. There are 18 channels: `Gift` ("Need-based help, with no debt kept"), `Share`, `Allocation`, `Inherit`, `Barter`, `Sale` ("Goods for the settlement's commodity money"), `Owner`, `Wage`, `Rent`, `Levy`, `Relief`, `Take`, `Restitution`, `Compensation`, `Fine`, `Bribe`, `Dues`, `Agreement`. None lends, repays or pays interest. A conservation check runs in tests and smoke seeds (ADR-0006 §3).
- **Money is inferred, per market** (`market.rs`, `Market::money`). Each settlement's market keeps, per good, the worth in sellers' hours of the payments settled in it, halving every 30 days (`memory_days`). Its money is the good that settles at least half that worth over at least 10 remembered trades (`money_share` 0.5, `money_min_trades` 10 in `content/core/people/early_farmers.toml`). "Nothing in the engine names a money good"; a settlement may never have one. Firewood became one test village's money within two months; the M3a demo found "no money anywhere" (plan §9, slice I and the M3a demo).
- **Posted terms are hours turned into goods** (`population/market.rs`, `post_terms`). A seller's ask is in hours of its own work (`ask_h`): its own cost times 1.25 (`margin`), times exp(−0.5 × spare years) for food, reviewed weekly (`review_days` 7) by at most 5 % (`max_change`). The posted price in a payment good is `ask_h` divided by the seller's own cost of that good. A payment good the seller has no own cost for is never accepted (`costs.get(p).copied().flatten()?`).
- **Own cost** (`value.rs`, `own_costs`) is the cheapest of gathering, growing (carried by the store's half-life, so grain is dearest before harvest) and making by a recipe a member knows; "`None` where it has no way to get the good". ADR-0019 §5 lets an offered good's anchor fall to its replacement cost from a price report.
- **Firms** (`firm.rs`, `pub struct Firm`). Only the household workshop: an owner, stores, offers, a `WageOffer` (pay good, units an hour, `hour_h`, reviewed every 14 days by at most 5 %), books (ring of entries, monthly statements), exits `Idle`, `OwnerGone`, `Failure`. "With no credit or debts in M3a, a wage that cannot be paid closes a workshop at once ... Revisit with credit" (plan §9, "Why workshops close").
- **Obligations** (`crime.rs`, `pub struct Obligation`; ADR-0015 §5). Kinds `Demanded`, `Restitution`, `Compensation`, `Fine`, counted in kcal of food; a debtor, a beneficiary, a due day, and a standing: `Open`, `Met`, `Refused` (unwilling), `Defaulted` (unable: "the rest is an arrear"), `Lapsed`. "An award never creates goods." ADR-0020 §7 adds obligations with a polity as debtor, carried by the store's keeper (`population/payments.rs`).
- **Ties** (`ties.rs`). Evidence per domain, one of which is `Word`: "Keeps agreements: wages, rent, what was lent". A tie also keeps `help_h`, hours of help received less given, which fades (a 1,095-day half-life in the code's core values): a reciprocity balance, not a debt.
- **Help and the polity's store.** A household short of food asks one holding more than twice its target; "No debt is kept" (plan §9, "Asking for food"). A polity's common store takes a share of threshing (`Levy`) and gives `Relief` to askers, under a keeper's roof by `KeepStore`. It pays nobody: public works are equal shares of labour, given or not. A field can be let for a share of its grain (`Rent`; ADR-0007 §3).
- **Price reports** (`reports.rs`). Per household, per market, good and payment good, halving in 30 days, dropped at 120, told at the hearth at 0.15. A household fetches to resell by an `Errand`.
- **Wealth measures** (`wealth.rs`): land held, worked, let and rented; goods in hours at the settlement's median own cost; floor area. "Income, consumption, debts ... are not measured yet" (plan §9, "Wealth measures").
- **Content**: 30 goods and 20 techniques, a spear among them, no metal and no metalworking. Nothing can be minted (the M5 trade brief §1.8; plan §9, M5 design).

## Where the reports agree

Eight points recur across 08-06, 08-07 and 08-20, with support from 08-04, 08-05, 08-11, 08-13 and 08-15. I treat them as invariants.

1. **No sequence of stages.** Barter, reciprocal obligations, commodity payments, coins and credit coexist and can revert (08-06, executive recommendation). Credit predates coins by many centuries (08-06 §1.1, §3), and "Credit can expand without deposit banks" (08-07 §4). Enable a capability by what agents built, never by a date (08-20 §5.1; 08-06 §5.5).
2. **Help is not a loan.** Keep a reciprocity record (relationship, remembered help, no exact balance) apart from a debt contract (creditor, debtor, principal, unit, settlement media, maturity, interest, collateral or guarantor, enforcement) (08-06 §1.2; 08-07 §1.2; 08-11 §1.1's *osotua* against *esile*).
3. **Credit moves timing, not goods.** Each loan has a purpose, a source of repayment and a timetable; credit confers no productivity bonus (08-07 §1.1). Borrowing is not income (08-15 §1.8; 08-13 §1.7).
4. **Lending depends on information and enforceability, not wealth alone.** Lenders see imperfect signals, ration rather than lend at any rate, and hold separate beliefs, never a visible credit score (08-07 §1.3, §5.3). No group gets a trustworthiness bonus (08-07 §1.3).
5. **Money is acceptance, and coin sells verification.** A holder accepts a good for use or for onward exchange, valued separately, less carrying, spoilage, verification and theft costs (08-06 §1.3). A coin issue is mass × fineness ÷ accounting units, and a new ordinance makes a new cohort, never a change to coins already struck (08-06 §1.5).
6. **Seigniorage and debasement are transactions.** The ruler must get metal, run a mint, spend the issue and face acceptance; winners and losers come from those transactions, "rather than applying a uniform wealth penalty" (08-06 §1.6). "Do not connect 'percentage debased' directly to 'percentage inflation'" (08-06 §1.10; 08-20 §1.3). There is "no defensible universal debasement-to-inflation coefficient" (08-06 §6).
7. **Accounting is stricter than behaviour.** Every claim has its matching liability; a transfer moves a claim, never copies it; a haircut changes both sides; no reserve multiplier (08-06 §5.3; 08-07 §5.2; 08-04 §4.1). Constant money is not an invariant, but creation and write-off need explicit entries (08-04 §4.1).
8. **Crises are failures of functions.** Insolvency and illiquidity differ (08-20 §1.4). A run comes from balance sheets and expectations (08-07 §1.9). Defaults come from household outcomes, never an "independent default die" (08-07 §2.3). Common exposure is not contagion (08-20 §1.5).

**What games and models teach to avoid:** a global popularity flag for money (08-06 §1.3); fixed reserve multipliers (08-06 §5.3); a permanent global blacklist after default (08-07 §4); "free, fully equipped clones" replacing bankrupt firms (08-04 §4.2); fixing a slump "by weakening conservation checks, granting free credit" (08-04 §4.6); Eco's permanent mint backing as a model of debasement (08-06 §5.6).

**Where they differ, or differ from the plan:**

- **The money ladder.** Plan §5.3 and §9 ("Money: discovered (barter → commodity → coin → credit → paper → fiat)") order the nodes. 08-06's executive recommendation and 08-07's opening reject a universal sequence, and 08-06 §4 tests that "a settlement with reliable records and enforcement but no mint should still support deferred commodity obligations". I follow the reports: credit in goods comes first and needs no coin.
- **Exchange rates.** Plan §5.3 has them "emerge from trade balance and trust". 08-06 §1.8 anchors two metal currencies at their fine-metal content, with a band for transport, assay, minting, restrictions and delay. The two agree only if "trust" means each holder's valuation; I keep the metal anchor.
- **Currency per polity.** Plan §5.3 gives each minting polity its own currency. 08-06 §1.4's Sichuan case shows tax receivability and regional circulation tied together, not one money per state. I keep coins per issuer and acceptance per market, as ADR-0019 §4 does.
- **Build order.** 08-20 §5.4 asks that "harvests, household access, stocks, obligations, and transport" work first, credit detail after; 08-04 §4.6 puts institutional finance at stage 5. TCE's grain asks barely answer scarcity, and the M4 dashboard still fails a food-price row (README; the M5 trade brief §3). Credit tied to that food economy may amplify its faults.
- **Interest.** 08-06 §2.1 gives 20 % (silver) and 33⅓ % (barley) as Mesopotamian conventions, medium confidence; 08-07 §2.1's table runs from 4 % ceilings to 25–35 % lending and says "Do not average this table". No report gives a rate for early farming villages (08-07 §3.1, §6).
- **Maturities.** 08-07 §2.3 tests merchant advances of 1–12 months; 08-20 §2.2 merchant credit of 30–180 days. Both are design priors.
- **What a temple is.** 08-07 §1.6 warns that "temple" is not evidence of a deposit bank; 08-20 §5.1 lists "Government, temple, or relief institution" as one crisis actor. TCE has no temple; its polity store is the nearest thing.

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| A loan | A claim: creditor, debtor, good lent and good owed, amounts, made and due days, increment, pledge or guarantor, standing, payments | A household's need and a lender's choice |
| Default and remedy | The claim's standing (ADR-0015's), and any finding with its bundle | The debtor's answer; the creditor's demand; a gathering's law |
| A pledge | The claim names a field's use, goods set aside, or a guarantor's promise | Agreed at lending; taken on default |
| Custody and deposits | A keeper's deposit claims, per depositor, beside its own stores | Households choosing to deposit; the keeper choosing to lend |
| A coin | Purse counts per cohort; a cohort record: issuer, day, mass, fineness, type | A mint recipe worked under a polity's law |
| Money's worth to a holder | Per household, a dated belief of what a unit of its market's money buys, in its own hours | Its own purchases and sales |

**Derived, not saved:** "bank", "moneylender" and "money changer" labels; exchange rates between coins; any price index; arrears totals and the dashboard's measures.

**Size (my arithmetic).** 3,000 people at five a household is 600 households. At 4–32 credit relationships each (08-06 §2.3), that is 2,400–19,200 claims; at an assumed 64 B a claim, 0.15–1.2 MB. Coins kept as purse counts per cohort, not objects (08-06 §5.4; 08-20 §5.2), cost a few numbers a household.

### 1.1 Two records: help and debt

**Reports.**

- A promise to help a relative is not a loan with principal, maturity and rate (08-07 §1.2). Need-based transfers do not create repayment debt; reciprocal ones do, and treating both as loans "would erase the defining difference" (08-11 §1.1, §5.4).
- Early farming credit: "seasonal commodity advances, household creditors, witnesses, storage-linked claims"; do not assume later Mesopotamian institutions in the first villages (08-07 §3.1).

**My proposal.**

- **Keep the gift path as built.** Asking for food stays help, recorded in `help_h` and in `Provision` evidence.
- **A loan is a second, explicit path:** a claim of an appended kind on ADR-0015's obligation record, generalised from kcal to a good and an amount. It moves on two appended ledger channels, *lent* and *repaid*, so the conservation check covers both legs.
- **Witnesses, not writing.** A claim is known to the two households and to whoever saw it made (a member at the hearth). Writing waits; 08-06 §5.5 does not require it "for every remembered obligation".

### 1.2 Who asks, who lends, and on what terms

**Reports.**

- Requested advance = max(0, needs before receipts − own resources − other financing) (08-07 §1.1).
- A lender's reservation rate: i_min = (cT + F/P + p(1 − R)) / ((1 − p)T), with p the believed default chance and R the recovery (08-07 §1.3). Higher rates can change who borrows, so lenders ration (08-07 §1.3).
- Search a bounded set: relatives, neighbours, trading partners (08-07 §5.3). Priors: seasonal household loans 0.25–1 year, tied to harvest (08-07 §2.3).
- Lending 100 kg of grain for 133⅓ is 33⅓ % in grain; if grain's price falls to 70 %, the money return is about −6⅔ % (08-07 §2.1, illustrative).
- Liquidity lets farmers store instead of selling at harvest and buying back later; one experiment found a 29 % return (08-05 §1.2, §3.1; not a universal rate).

**My proposal.**

- **Need has a purpose.** A household asks for a loan when its plan to the next harvest falls short: seed before sowing, food beyond what help gives, a tool, or the payment for an errand (ADR-0019 §6). Each request names what it is for and when it can repay (the harvest, an errand's sales).
- **Lenders are households and workshops with spare.** The candidates are those its members hold ties to, then any who hold more than they keep (the gift rule's test), within the trade walk. A workshop lends its own stock (08-07 §1.6's own-funds moneylending).
- **The lender decides by 08-07 §1.3's rule, from beliefs.** p comes from its ties' `Word` evidence for the asker's household, and from the harvest it expects for that household; R from the pledge offered; c is what the good would earn or lose kept at home. Below i_min, or above a believed p, it refuses.
- **Terms are a commodity increment, due at a date.** Lent in one good, owed in the same good or another the lender names, with an increment. A conventional increment is a content prior per world (open question 3), not a rate by era.
- **Illustration (my arithmetic).** T = 0.5 year, p = 0.1, R = 0.5, c = 0.05, F = 0: i_min = (0.025 + 0.05)/0.45 ≈ 17 % a year, or 8 % on the half year. Separately, roofed grain is asked about 4 log points dearer before harvest than after (plan §9, slice L): a spring kilogram repaid with an autumn kilogram loses about 4 % in hours before any increment.
- **Interest is measured, never set.** The dashboard reports realised lender returns after losses, as 08-07 §4 asks.

### 1.3 Default, pledges, remedies and debt bondage

**Reports.**

- Collateral must be identifiable, pledgeable, seizable and saleable; ceiling = sale value × recoverable share − senior claims − collection costs (08-07 §1.4). Priors: advance 30–70 % of conservatively valued collateral; recovery 20–80 % (08-07 §2.3).
- 71.43 % of one sample of 336 merchant loans relied on guarantors alone (08-07 §1.4, §2.2). Guarantees are contingent obligations, and one guarantor can link failures (08-07 §1.4).
- Arrears have causes: inability, liquidity in the required medium, dispute, concealment, refusal (08-13 §4.1). Default can be a standstill, an extension, a haircut or seizure, and need not end lending (08-07 §1.8, §4; 08-20 §1.4). Hammurabi §48 relieves grain repayment in a failed harvest (08-20 §1.6).
- "Do not infer permanent bondage merely from debt" (08-10 §1.1). Coercive relationships are separate from occupation and personality (06-10 §4.A). Pawnship and coercive credit existed in particular African settings and need region-specific rules (08-07 §3.2).

**My proposal.**

- **Pledges at village scale are things people already hold:** goods set aside under the lender's roof (custody, not ownership); the use of a field until repaid, on the existing lease (`let_ha`, `Rent`), the creditor working it or letting it; or a guarantor's promise. Land is not sold in TCE (ADR-0007 revisit), so no mortgage.
- **At the due day the debtor answers,** as it answers a finding today: pay, pay part, ask for time, or refuse. `Defaulted` and `Refused` keep their meanings. The creditor may grant time, take the pledge, call on the guarantor, or demand (a visit with a purpose, as for takings).
- **A gathering hears debts only by law.** An appended policy kind, *debts heard*, whose bundle the custom chooses: extension in a lean year, the pledge taken, labour-days owed to the creditor (ADR-0015 §5's existing kind), or nothing more. Without it, a creditor has its own answers and its `Word` evidence.
- **Debt bondage.** The engine authors no unfree status. Labour-days owed are finite, counted, and leave the person's household, residence and choices intact (06-10 §4.A). A gathering could pass harsh bundles; that is its choice, recorded. Pledging a person waits (§2).
- **Default teaches.** A missed payment writes bad `Word` evidence for the debtor's household in the creditor's ties, and travels by word (08-07 §5.3: "not teleport").
- **A wage arrear.** A workshop that cannot pay a wage owes it as a claim, and fails only when it cannot pay by the due day (plan §9's "Revisit with credit"; 08-04 §4.1: assets at failure are transferred, never vanish).

### 1.4 Lenders, keepers and what a bank is

**Reports.**

- Six businesses: own-funds lending, safekeeping, deposit intermediation, transfer banking, bill acceptance, partnership finance (08-07 §1.6). A pure custody institution holds 100 % (08-07 §2.3). "Reserves are not capital" (08-07 §1.6).
- A loan creates a deposit only where the lender's liabilities are accepted for payment; a nonbank's loan transfers an existing balance (08-06 §1.9; 08-07 §1.6).
- A warehouse receipt, a redeemable note and a transferable deposit are distinct instruments (08-06 §1.9). Credit becomes money-like only when third parties accept assignment (08-06 §1.2).
- Public lending: Rome in AD 33 lent for 3 years at 0 % against land worth twice the loan (08-20 §2.1). Relief can lower local self-insurance (08-20 §1.6).

**My proposal.**

- **No bank is authored.** A bank is a derived label for a holder doing 08-07 §1.6's things, as polity labels are derived.
- **Custody first.** A household with roofed room (`storage_kg`) may keep others' grain for a fee in grain. The deposit is the depositor's claim; the grain sits in the keeper's store on an appended custody channel, never counted as the keeper's own (08-06 §5.3: no double spending of backing).
- **Lending deposits is the keeper's choice.** Once it lends deposited grain, it owes more on demand than it holds: deposit intermediation, with its run risk (§1.9). Whether a keeper may do so is a law's question (open question 5).
- **The polity's store may lend.** An appended policy kind, *lend from the store*: seed or food lent from the common store at terms the law sets, repaid into it, carried by the keeper. Relief stays a gift.
- **No receipts that circulate in M7's first slices.** A transferable claim needs assignment rules and acceptance (08-06 §5.5), which the M5 trade brief §1.8 also declined to invent.

### 1.5 What money is worth to its holder

**Reports.**

- Keep a local acceptance estimate from successful and refused payments (08-06 §1.3). Desired transaction balances: 7–30 days of forecast spending (08-06 §2.3). Retail repricing every 1–7 days, contract wages every 30–180 (08-06 §2.3).
- Quote in an accounting unit; the payer chooses what to pay with by opportunity cost; the recipient values it by local quotes and quality (08-06 §5.2).

**What the code implies (checked above).** A seller values a payment good at its own cost of getting it. That suits firewood or grain. A coin has no own cost to anyone but a mint, so no seller would accept one. And because asks are anchored in hours and move at most 5 % a review, more money of any kind could move posted prices only through the money's worth in hours.

**My proposal.**

- **Once a market has a money, its worth to a holder is what it buys.** Each household keeps a belief, in its own hours, of what a unit of its market's money fetched in its recent purchases, fading by `memory_days` (08-06 §1.3's onward exchange, estimated apart from use). The holder values a unit at the greater of its use value (its own cost, as today) and this belief, less what verifying it costs.
- **Beyond a desired balance a unit is worth less,** by the rule stores already use, target ÷ (target + held) (plan §9, "Diminishing value of stores"), the target being 7–30 days of its money spending (08-06 §2.3). A seller holding many coins asks more of them; one holding few asks fewer.
- **This is the only route from the quantity of money to posted prices.** Prices in hours keep their anchors; prices in money move as holders' beliefs of money's worth move. The engine reads no index.

### 1.6 How coin comes to exist

**Reports.**

- Capabilities, not eras: weighed-metal settlement needs scales, metalworking and verification; coin needs standardized production, recognizable marks and a trusted or enforced valuation (08-06 §5.5).
- Minting consumes metal, fuel, labour and equipment; people surrender bullion when coin is worth the fee, or under coercion (08-06 §1.5). Net mint revenue = coins produced − metal paid − minting costs (08-06 §1.6). Priors: net mint charge 0–10 % (08-06 §2.3). English seigniorage was usually under 2 % of state revenue, 57 % in the Great Debasement (08-06 §2.1).
- Accepting a medium for dues creates demand for it (08-06 §1.4). A household owing a due in one medium seeks it, and wealth is not liquidity (08-06 §1.4; 08-13 §1.5).
- "Collecting coins transfers ownership to the treasury. It does not physically destroy them" (08-06 §5.3). A large polity can run without retail coin (08-06 §3, Inka).

**My proposal.**

- **Metal first, as content.** An ore, a smelting technique and a metal good, found and spread as techniques are (ADR-0008). Metal may be weighed and paid by weight before any coin, through the existing acceptance tally.
- **A mint is a recipe** (metal and fuel in, coins of a cohort out) gated by a technique and by a polity law naming an office with the power to mint (plan §5.2). Its holder works it, as the keeper keeps the store.
- **A cohort is saved:** issuer, day, mass, fineness, a mark. Purses hold counts by cohort.
- **The issue enters by transactions:** households bring metal and take coins less the law's fee; the store buys metal; the polity pays in coin for work it asks (an appended choice: paid shares of public works, a hired band, a carrier). Today a polity pays nobody, so without spending a mint has nothing to do.
- **A levy may name its medium.** A law can ask the levy in coin. Households then must sell grain for coin, often at harvest (08-05 §1.2's distress sales).

### 1.7 Debasement and prices

**Reports.**

- Debasement, retariffing and redenomination are different (08-06 §1.5). Cutting fine metal per unit by 25 % yields about 33⅓ % more units, not 33⅓ % higher prices (08-06 §1.6). The Ottoman akçe lost 44 % of its silver in 1585–86: 1/0.56 = 1.79 is an accounting benchmark, "not a prediction" (08-20 §1.3, §2.1).
- Gresham's law holds when coins of different metal discharge the same obligation at the same value; where coins trade at different prices or by weight, qualities coexist (08-06 §1.7). Old and new issues coexist (08-06 §4).
- France 1724: exchange rates answered quickly, goods and wages slowly and incompletely (08-06 §1.10). Inflation runs through new spending, suppliers' price decisions, wages and contracts on their own schedules, and holders' changed balances (08-06 §1.10).
- Grain debts are untouched by inflation; money debts are not (08-20 §1.3). France debased 123 times in 1285–1490, 112 over 5 % (08-06 §2.1).

**My proposal.**

- **Debasement is a mint law:** the next cohort's mass or fineness, and the fee. It never touches coins already struck.
- **How a holder tells cohorts apart is a skill.** One who can assay (a metalworker) values by metal; others value by count until their purchases teach otherwise. Paying, a household gives the cohort worth least to it (08-06 §1.7). Gresham then follows from who can tell, not from a rule.
- **Prices answer through §1.5.** The mint's extra coins are spent by the polity at the first sellers it pays. Their holdings pass the desired balance, so they ask more coins; their purchases teach others that a coin buys less. Asks review weekly and wages every 14 days, so wages lag (08-06 §1.10). Claims in grain do not move; claims in coin lose in hours.
- **Who gains and who loses is measured,** by household, as 08-06 §4 asks: the polity, those paid first, debtors in coin, sellers who can assay.

### 1.8 Currencies between villages

**Reports.**

- A money changer has inventories, quotes, assay and correspondents; two metal coins trade near their fine-metal parity within a band (08-06 §1.8). Spreads: 0.5–5 % (08-06 §2.3).
- Keep quotes only "where actual trading relationships exist" (08-06 §5.4). Financial integration is uneven by route (08-07 §4).

**My proposal.**

- **Acceptance stays per market** (ADR-0019 §4). A seller in A values B's coin by what it has bought with it, if anything, else by weight, less its assay cost. Foreign coin trades by weight, at a discount, until people use it; that discount is the emerging exchange rate.
- **Price reports carry the cohort.** A report says "3 coins of B's second issue", and a fetcher values it by its own belief.
- **The observer derives the rate** from trades that paid in each coin, in hours. Nothing reads it.
- **Money changers wait** until households hold several coins in quantity (§2).

### 1.9 Crises as mechanisms

**Reports.**

- Harvest-linked debts synchronize repayment demand, and crop failures correlate defaults (08-06 §1.2). Fifty farms on one water system are not fifty independent risks (08-07 §5.3). A fixed obligation takes 40 % of disposable food after a 20 % harvest loss, in one illustration (08-20 §1.1).
- A run occurs when demands exceed reserves, near receipts, emergency funding and sale proceeds; fear that others withdraw first can bring down a viable keeper (08-07 §1.9). Payment order in a run is "an institutional rule—not an accidental consequence of memory order" (08-07 §5.6).
- Three channels: direct default, shared collateral, funding confidence (08-20 §1.4). 1763: grain prices fell 30 % and 38 firms failed, many reopening within months (08-20 §2.1).

**My proposal.**

- **Correlated default needs no rule.** A world has "a daily weather series per landscape" (plan §7, slice U), so neighbours' harvests fail together, and claims due at harvest fall due together.
- **A run is depositors' choices.** A depositor who hears that the keeper lent grain, or that a debtor of the keeper defaulted, weighs withdrawing; one who sees others withdraw weighs it more. The keeper pays in an order its law sets, or first come by a seeded order.
- **Collateral falls with what it is.** Pledged grain is valued at asks; a glut after harvest lowers it. Field use falls with yields.
- **Recovery is a vector:** arrears, lending volume, deposits, prices in hours and in coin, each tracked apart (08-20 §1.6, §4).

## 2. What to defer, and why

| Defer | To | Why |
|---|---|---|
| Circulating receipts, notes, bills of exchange | M8 or later | Assignment, authentication and acceptance networks (08-06 §5.5; 08-07 §1.7) |
| Money changers as firms | After several coins circulate | 08-06 §1.8 needs currencies in quantity |
| Partnerships, profit shares, annuities | With firms beyond the household workshop | 08-07 §1.5 keeps their payoffs distinct |
| Public debt and polity default | After polities spend | 08-07 §1.8 needs revenue to pledge |
| Pledged persons, pawnship | Not without a design for custody | 08-07 §3.2; 06-10 §4.A; ADR-0023 left custody of captives to M7 |
| Land mortgages | With land sales | ADR-0007 revisit; 08-07 §1.4 |
| Hyperinflation scenarios | Never as a status | 08-06 §1.10: a feedback loop, not an effect |
| A credit or inflation god tool | Never | It would choose an outcome |

## 3. Risks and validation

**Plot.** No date, era, population or wealth opens credit or coin; no issue weighs toward a loan or a mint; no interest by era (08-07 §2.1); no inflation coefficient (08-06 §6); no index read by behaviour; no default die (08-07 §2.3); no rescue that creates goods (08-20, core recommendation). If nobody lends after a week, nudge a prior (the increment, the trust that `Word` evidence gives), never a loan, and log a `NUDGE:` (plan §1, rule 3).

**Stability.** Credit can turn a lean year into a cascade (08-04 §4.2). Measure before tuning: requests, refusals and why, loans made, repaid, extended and defaulted, pledges taken. Build credit on the food economy as it is, and say so if the food-price row's faults pass through (08-20 §5.4).

**Stylized facts, as plausibility checks:**

- Credit without coin or bank: lending runs in a world with no metal (08-06 §4; 08-07 §4).
- Wide rate dispersion within one village, by borrower, pledge and tie (08-07 §4).
- A cap that binds changes who borrows and with what pledge, not only the rate (08-07 §1.5, §4).
- Bad harvests outnumber famines; credit sometimes bridges and sometimes deepens a shortfall (08-20 §4).
- Old and new cohorts coexist after a debasement; coin prices move by goods at different speeds, wages last (08-06 §4).
- A monetary change without a harvest change, and the reverse, look different in hours (08-20 §1.3, §4).
- Common exposure is not contagion: replay a lean year with lending switched off as a test harness (08-20 §1.5; the M5 trade brief's twin).
- Lenders neither all fail nor all grow rich (08-07 §4).

**Observer.** A household's page shows what it owes and is owed, to whom, due when and why. A keeper's page shows deposits, what was lent and what is on hand. Each market shows prices in coin and in hours side by side (08-20 §1.3), cohorts in circulation, and refusals. The chronicle tells defaults, runs and mint laws in plain verbs.

**Compute.** Claims are sparse and due on dates: a maturity queue, not a daily scan (08-07 §5.6; 08-20 §5.2). Lending is weighed only by a household with a shortfall. A world with no loans must give the build-before's digests.

**Choices expensive to reverse.** One ADR, "Claims and coin": claims as holdings (ADR-0006's revisit trigger), the obligation record generalised from kcal to goods, custody and its channel, appended ledger channels, coin cohorts and purse counts. Coin could be split out if metal content slips.

## 4. Suggested slices

1. **Loans in goods between households:** the claim, the asking and lending choice by 08-07 §1.3 from ties, harvest due dates, repayment, and `Word` evidence from what was repaid.
2. **Default and pledges:** answers at the due day, field use and goods as pledges, guarantors, the *debts heard* law with finite labour-days, and wage arrears for workshops.
3. **Keepers and the polity's store:** custody for a fee, lending deposits by the keeper's choice with runs possible, and *lend from the store*.
4. **Money's worth to a holder:** purchasing-power beliefs and the desired balance for the inferred money, checked against today's markets before any coin.
5. **Metal, mint and debasement:** metal content, the mint recipe and law, cohorts, polity spending and a levy in coin, then foreign coin by weight between villages.

## 5. Open questions for the designer

1. **Order.** Credit in goods first, as the reports suggest, against the plan's ladder (coin before credit)?
2. **Metal.** Which metal, in which milestone? Without it, M7's "coin debasement and inflation" cannot happen and its demo needs none.
3. **The increment.** Is a conventional increment a content prior per world, set by each lender from i_min, or both?
4. **Record.** Generalise ADR-0015's obligation (kcal) to goods, or add a separate claim table?
5. **Deposits.** May a keeper lend deposits by its own choice, or only under a law?
6. **Bondage.** Are finite labour-days the harshest default bundle the content offers in M7?
7. **Money's worth.** Replace own cost by purchasing-power beliefs for an inferred commodity money too (firewood, grain), or only for coin? The first changes every world and needs a dashboard rerun.
8. **Polity spending.** Paid shares of public works, hired bands or carriers: which first, so that a mint has a use?
9. **Assay.** Who can tell cohorts apart: metalworkers only, anyone with scales, or anyone after use?
10. **Exchange rates.** Is a metal-parity band with a weight discount acceptable for plan §5.3's "trade balance and trust"?
11. **Measures.** Which defaults, runs and price paths go on the dashboard, and with what tolerances?
