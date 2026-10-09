# M5 design brief: trade between settlements

**Scope.** This brief covers:

- who carries goods between settlements, what they carry and why, how they decide and what they risk;
- how two settlements' prices come to converge, or fail to, what a plausible convergence looks like and how to measure it;
- markets that serve several settlements;
- money between settlements, and per-polity currencies once minting exists;
- the dashboard row and the demo measure for price convergence (plan §7, M5 demo: "two settlements trade, their prices converge").

Roads, bridges, diplomacy and the diffusion of technique and style are other briefs; this one names where they touch trade. The scale is 3–8k people in 2–3 settlements (plan §4.3). Worlds are 2–16 km a side (plan §9, world sizes), and a family sent more than 600 m from a lived-in settlement founds its own (`SPAWN_JOIN_M`), so neighbours can be anywhere from under a kilometre to a day's walk apart.

**What exists today (checked in code).**

- Each settlement has one market (`kernel/crates/civ-agents/src/market.rs`). Its tallies fade by half in 30 days, its price history records what payments were worth to sellers in hours of their own work, and its money is inferred: the good that settles at least half the payments' worth over at least 10 remembered trades.
- A buyer searches only sellers of its own settlement (`civ-agents/src/population/market.rs`). The trade activity allows at most a 30-minute walk (`content/core/activity/trade.toml`). That is 1.5–2.5 km at the content's 3–5 km/h, off trail and on it (my arithmetic). Payment is carried by the buyer and settles at the seller's door.
- **Asks.** The anchor is own cost × 1.25 × exp(−0.5 × spare years). Each weekly review moves half the log distance toward the anchor. It moves a further half of the 5 % cap down when nothing sold in the settlement, or up when demand went unmet (`reviewed_ask`). At steady state an ask therefore sits within about ±5 log points of its anchor (my arithmetic from the update rule).
- **Own costs** are in hours of a capable adult's work (`value.rs`), so they compare across settlements.
  - Grain's cost comes from the content's crop parameters, carried by its storage half-life. It is the same for every household in a world on a given day, except for whether the household's store is roofed.
  - A tool's cost follows the household's skill, the techniques it knows and the places it gathers from.
- Unmet demand is recorded only for tools nobody offers (README, "A small market"), so a grain shortage never nudges asks up.
- Each world has one daily weather series (README), so neighbouring settlements' harvests are correlated.
- Core content has no metal good and no metalworking technique (24 goods, 14 techniques). Nothing can be minted.
- A river with a mean discharge above 3 m³/s cannot be waded (content `[walking]`), so some neighbours need a bridge before they can trade.
- I found no activity that keeps a person away from home overnight.

## Where the reports agree (invariants)

1. **Shipments, local markets and beliefs, not a global price plus a distance surcharge** (08-12 opening and §5.3; 08-05 opening; 08-15 §1.7). Physical truth resolves moves and sales; beliefs choose them.
2. **Gaps close to a no-arbitrage band, not to zero.** A gap smaller than the cost of carrying a good can persist with no trade (08-05 §1.7; 08-12 §4). There is no universal convergence speed and no town-convergence half-life to start from (08-05 §1.7, §3.2).
3. **Merchants know dated reports, not today's prices elsewhere.** Information moves apart from goods (08-12 §1.6, §5.3; 08-05 §1.4, §6).
4. **Goods in transit stay owned and counted.** Arrival is not a sale (08-12 §5.5; 08-05 §6).
5. **Tradability emerges from the goods,** from value per kilogram, perishability and scarcity at the destination, never from "local" or "luxury" flags (08-12 §1.2; 08-16 §1.2).
6. **Money is acceptance, not a flag.** Coins are minted from real inputs, and seigniorage is a transaction (08-06 §1.3, §1.5, §1.6).
7. **Price correlation is not integration.** Shared weather, seasons and money move disconnected towns together (08-12 §4; 08-05 §7).

**Where they differ.**

- **Emphasis.** 08-05's closing recommendation starts from periodic venues. 08-12 §5.6 starts from real cargo, finite markets, directional transport, provisioning and delayed information.
- **Search size.** 2–5 known sellers (08-05 §3.2), 4–16 counterparties (08-06 §2.3), 8–32 candidate markets for a merchant (08-12 §2.4). With 2–3 settlements the difference is moot.
- **Trust.** Multilateral reputation and formal enforcement are both contested readings; neither may be hard-coded (08-12 §1.3; 08-05 §7).
- **Units of the seasonal gap.** 08-05 §3.1 reports maize's 33.1 as a percentage gap, 16-01 §2.2 as log-percentage points (a 39.2 % premium), and 08-15 §2.5 "in the study's reported convention". The dashboard reads 16.6–60.8 as log points.
- **Exchange rates.** Plan §5.3 has them emerge from trade balance and trust. 08-06 §1.8 anchors two metal currencies at their fine-metal content, with a band for transport, assay and delay.

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| Contact and reach | Per household, markets it knows of: settlement, first heard, source | A member visits, trades there, or hears of it at home or the hearth (ADR-0016 §3's contact rule) |
| Price reports | Per household, the latest per market and good: ask in hours, units offered, wanted, day seen, seen or told, from whom | Trading or standing at a market; hearing a report |
| Fetch trips | Per trip: carrier, household, the load (a ledger holder), origin, destination, departure | A purchase chosen from another market; fetching to resell |
| Household merchant, then firm | Existing firm record and books; fetched goods enter as purchases | The household's weekly review, as making to sell does today |
| Market tallies | Existing per-settlement market, plus each trade's buyer settlement | Every trade settled at a seller's door |
| Money | Unchanged: per-market acceptance. Later, coins with an issuer and a cohort | Payments; a mint (deferred) |
| Convergence record | Per settlement pair and month: median asks and realised prices of goods both offer, kilograms carried, walk hours | Monthly, from the markets |

**Derived, not saved:** hearth-to-hearth routes, cached monthly by origin and destination (08-12 §5.2: "do not let every character solve every route"); the carry band per good.

**Size:** price reports for 1,600 households (8,000 people at five a household) × 3 markets × 24 goods × 32 B ≈ 4 MB (my arithmetic). Keeping them per household rather than per person is my proposal; ADR-0016 holds hearing per person.

### 1.1 Contact and reach

**Reports.**
- Households compare nearby outlets; merchants connect outlets to suppliers (08-16 §5.3).
- Networks combine local clusters and distant links; avoid an automatically complete trade graph (08-12 §4, last row).
- Knowing that another town is cheap does not make its goods available (08-05 §6).

**My proposal.**
- A household can buy from another settlement's sellers only when it holds a report of that market.
- Reports start with a person who was there: trading, hunting or gathering nearby, a family that came from there, or kin after splinter founding.
- A separate content activity, *fetch*, has a longer walk limit than *trade*, so in-settlement trade stays as tuned. The limit is a tuning value, bounded by daylight while nobody stays away overnight.
- Below about 2 km the two markets simply merge into each household's reach, and that is the honest outcome.

### 1.2 Who carries goods

**Reports.**
- Merchants, carriers, investors and agents are separable roles. A household merchant supplies capital, labour, storage and risk, and its absences cost its other work (08-12 §1.3).
- These are composable arrangements, not a sequence (08-12 §1.3).
- A convoy shares a fixed protection cost but takes time to assemble and eats more supplies (08-12 §1.4). A convoy aggregates movement while keeping each participant's identity (08-12 §5.1).

**My proposal, in order.**
1. **Buyers fetch for themselves.** Today's purchase rule, the offer that saves the most of the household's own hours with the walk included (plan §9, slice I; 08-05 §1.4), extends to known sellers elsewhere.
2. **Household merchants fetch to resell.** A household buys a load where it is cheap, carries it home and offers it at its own door through its normal terms.
   - This mirrors making to sell (plan §9, slice I), and fetched goods are its cost.
   - It needs no new venue: the importing settlement's people do the carrying, and the outward leg carries the payment, which is real return-load economics (08-12 §4, outbound and return differ).
   - The README's idle-hours entry already asks that trade draw on the 10 or so hours a day adults spend resting or at the hearth.
3. **Merchant firms.** A household that fetches repeatedly sets up a firm whose stock and books are its own, as a workshop is.
4. **Carry to sell** (a seller standing at the other settlement's hearth for a session, where buyers weigh its offer like any other) comes only if steps 1–3 leave gaps the band cannot explain.

**Caravans.** At M5 there is no road danger (war is deferred) and no pack animal (no livestock in content). The reports therefore give no economic reason for a convoy. My proposal is that a caravan be a derived view of fetch trips that share a route and a day, not a new actor. Joint travel may come from people's need for company, but that is a guess.

### 1.3 What they carry, and why

**Reports.**
- Freight burden ≈ c × distance ÷ value per unit mass (08-12 §1.2). Heavy cheap goods favour short legs; perishables need nearby buyers; scarce essentials can justify costly carriage (08-12 §1.2 table).
- Do not restrict trade to luxuries (08-12 §1.2).
- Derive early transport costs in labour units (08-12 §2.3). Its illustrative porter (25 kg, 25 km a day) needs 1.6 worker-days a tonne-km, or 3.2 with an empty return.
- Exportable surplus is what remains after household needs, seed, losses, target stocks and obligations. In its illustration a 20 % harvest fall wipes it out (08-16 §1.1). Offered stock is not physical stock (08-05 §1.2).

**In TCE units (my arithmetic).**
- **Grain.** A 20 kg load (content `carry_kg`) carried d km with an empty return costs d/50 hours a kg on a trail and d/30 off one.
  - At 5 km that is 0.10–0.17 h/kg. Against the grain asks of 1.1–1.9 h/kg in plan §9's runs, that is 5–14 log points.
  - At 10 km it is 10–26 log points.
- **Tools.** A single sickle asked 3.6–4.4 hours (M3a demo) costs a one-off buyer 2–3.3 hours to fetch from 5 km, 45–93 % of its price. A merchant's full load divides that cost.
- **Perishables.** Fish and meat, with half-lives of 2–3 days, will not travel.

My expectation, not a target: the goods that differ most between settlements move first:
- flint, stone, clay and reeds, which follow the places each settlement knows;
- tools and pots, which follow skill and known techniques;
- dried fish and meat;
- grain only when one settlement's sellers hold far more to spare. The stock term alone can open up to 49 log points between a seller with a year to spare and one with none (my arithmetic, 0.5 × 100).

### 1.4 How they decide

**Reports.**
- Expected profit = sale revenue − purchase − transport − fees − capital − insurance (08-12 §1.1). Revenue must reflect finite demand; multiplying a cargo by the last price creates artificial arbitrage (08-12 §1.1).
- Ship when (1 − loss) × expected destination price − source price exceeds freight, finance, tolls and risk, all in one unit (08-05 §1.7).
- Estimate both price and market depth, or every merchant answers the same shortage and gluts it (08-12 §1.6). Order against expected demand over the replenishment time (08-16 §1.3).
- Production specialization responds more slowly than trading (08-12 §1.6).

**My proposal.** At its weekly review a household weighs, for each good and known market, the expected margin of one fetch trip, in hours:

`margin = q × (p_home − p_there) − walk hours − trade minutes`

- q is the least of: a load; the units the report says were offered; and the units the household expects to sell at home over the next review, from its market's sales and unmet demand.
- p_there is the reported ask, payable in goods the household holds that the other market's sellers accept.
- Each report's weight halves with its age, by a half-life that is a tuning value.
- The trip then competes in the activity scorer with fields and leisure, as hired work does.
- No household reads another market's truth.

### 1.5 What they risk

**Reports.**
- Profitability is not liquidity: an owner may be unable to finance the purchase (08-12 §1.1).
- Hazards are route- and good-specific, and no universal loss rate exists (08-12 §6). The test priors are 0.5, 2 and 10 % over 50 travel days, turned into a daily hazard so that step size does not change risk (08-12 §2.4).
- Share one storm across a convoy rather than rolling for each unit (08-12 §5.5).
- Trust matters where cheating pays: gain from cheating < expected penalty + collateral + future business (08-12 §1.3).

**At M5 distances (my arithmetic).**
- A 2 % exposure is a hazard of 0.0004 a day, about 0.04 % for a one-day trip. Catastrophic loss is negligible.
- Financing at 15 % a year costs about 0.04 % a day, also negligible.

**The real risks are** hours away from the fields at the peaks; a stale report (the seller sold out, or others came first); payment goods carried out and back unspent; spoilage; a ford too deep for a while; and a glut at home after others fetched the same good.

**Default.** Spot settlement at the seller's door leaves nothing to default on. Trust becomes a mechanism only with credit or agents (deferred). My proposal: let ties (ADR-0014) break near-ties in the choice of seller, as relationships do (08-05 §1.3–1.4), and nothing more.

### 1.6 Prices: how gaps close, or fail to

**Reports.**
- Search costs sustain local differences (08-05 §1.4).
- A seller's reference price should incorporate procurement, carrying costs and expected scarcity, and production cost is not an absolute floor (08-05 §1.5).
- Integration narrows profitable gaps, not all gaps. Adjustment must come through purchases and arrivals, never a town-price synchronization function (08-05 §5 D).
- Profits attract entry but need not converge smoothly: delayed information and lumpy shipments cause oversupply, losses and exit (08-12 §4). Trade can buffer independent shocks and transmit correlated ones (08-12 §4).

**What the current controller implies (verified above).**
- Asks stay within about 5 log points of each seller's own-cost anchor. If B's own cost of flint is twice A's, B's sellers keep asking their cost and simply stop selling, while B's buyers and merchants fetch from A.
- Once B's people can reach A, what B's buyers pay would converge (A's ask plus the walk), but B's asks would not, except for food through the stock term (imports raise spare stocks, exports lower them).

**My proposal.**
- A household's anchor for a good becomes the lesser of its own cost and its replacement cost from the best market it has a report of: reported ask plus carry hours per unit. This is 08-05 §1.5's procurement term.
- Asks should then settle about a band apart, with no rule naming the other town's price.
- Whether this also applies to offers inside one settlement, which would narrow today's house-to-house spread, is an open question.

**Plausible convergence.**
- Goods whose gap exceeds the carry band trade until their gap is about the band. Goods inside the band do not trade, and their gap stays put (08-05 §1.7's 100/130/25 example).
- The band differs by good, with value per kilogram, and by direction: uphill against downhill, and the payment carried out against goods carried back (08-12 §4).
- Shocks reopen gaps for a while, because loads and hours are finite (08-05 §5 D), and an arrival glut can overshoot (08-12 §1.6).
- Both prices move, not only the importer's: for food, the exporter's spare falls and the importer's rises; with the procurement anchor, the importer's anchor falls for any good.

**Failure to converge is plausible** when there is no contact or report; when the walk costs more than the gap; when nobody offers, surplus being gone in a lean year both share under one weather series; when buyers hold nothing the sellers accept, or are hungry with nothing to pay (08-05 §1.2: hunger is not purchasing power; 08-16 §1.7); or when a river is unfordable before a bridge. Each needs a recorded cause, as 08-16 §5.6 asks of failed deliveries.

### 1.7 Markets that serve several settlements

**Reports.**
- Periodic markets concentrate demand. A venue is viable when expected margins cover attendance, travel and fees, and periodicity is an institutional choice, not a population threshold (08-05 §1.1).
- Skinner's remotest villagers lived 3.4–6.1 km from their standard market, and Chinese schedules ran 2–3 sessions per 10 days (08-05 §3.1).
- Supply regions follow transport cost, not circles (08-16 §1.2, §4).

**My proposal.**
- A market stays keyed by the settlement where the seller lives, and a trade is tallied where it settles.
- "A market serving several settlements" is then measured, not declared: the share of a market's trades bought by households of other settlements, and the reach of its buyers.
- A periodic market day is a policy template a polity could adopt through the law pipeline, attendance being each seller's choice by 08-05 §1.1's margin rule. I would defer it to M6's towns unless the demo needs it.

### 1.8 Money between settlements, and per-polity currencies

**Reports.**
- Ask why this person accepts this payment from this counterparty (08-06, executive recommendation). A medium useful between strangers can spread where internal distribution stays reciprocal (08-06 §1.1, Fauvelle).
- Grain serves local obligations but travels poorly, and a durable valuable can serve between communities (08-06 §1.3).
- Keep a local acceptance estimate, never a global flag (08-06 §1.3).
- **Coinage** sells verification. A coin issue is coin mass × fineness ÷ accounting units, and a new ordinance creates a new cohort (08-06 §1.5).
  - Minting consumes metal, fuel, labour and equipment, and people surrender bullion only when coin is worth the fee, or when coerced (08-06 §1.5).
  - Seigniorage is a transaction (08-06 §1.6).
  - Gresham's law is conditional (08-06 §1.7).
- An authority makes a medium useful by accepting it for dues it can enforce (08-06 §1.4).
- Money changers hold inventories and quote spreads (08-06 §1.8).
- Coin issues need standardized production, recognizable marks and a trusted valuation (08-06 §5.5).
- A large polity can run without a retail currency (08-06 §3, Inka).

**My proposal.**
- Acceptance stays per market. A visitor pays in what the seller accepts, and the payment counts in the seller's market.
- Two outcomes can emerge without a rule, both to be measured:
  - The same good becomes money in both markets: a shared commodity money.
  - Payments between settlements settle in a third good, light and durable (08-06 §1.3's portability).
- Per-polity currency waits for minting, and core content has nothing to mint. I would not invent a token or a store receipt to satisfy the plan line: a receipt is a transferable claim, which needs records and enforcement (08-06 §1.9, §5.5).
- When metal content arrives, the coin is a good with an issuer and a cohort, minted by a recipe gated by a technique and an office's `mint` power (plan §5.2). Its acceptance is the same per-market tally, and the polity may name it as payable for the levy.

### 1.9 What is expensive to reverse

- **ADR candidate A: exchange between settlements.**
  - Reach across settlements through reports.
  - Price reports as household beliefs with dates and sources.
  - Goods in transit as a ledger holder (the load), so they are counted, can spoil and arrive with their carrier.
  - Trades tallied where they settle.
  - It amends ADR-0006 §3–§4 and adds save sections.
- **ADR candidate B, only if metal content arrives: coinage.** Covers coin cohorts, the mint and the issuer. ADR-0006 already names this as a revisit trigger.
- **Not ADRs:** the fetch activity, report half-lives and the convergence record are append-only content and saves.
- **Codebase risks.** Long walks make route searches dearer, and they are already most of a slow day's time (plan §9), so cache hearth-to-hearth routes. Accelerated mode's household view holds "the purchase" (ADR-0011 §4), so Gate B must cover purchases between settlements. If trade has not appeared after about a week of tuning, author a nudge (a prior or weight) and log it as `NUDGE:` (plan §1, rule 3); never script a trip (rule 1).

## 2. Numbers worth keeping

| Quantity | Value | Source |
|---|---|---|
| Porter load | 15–30 kg net; about 20 kg (TCE `carry_kg` 20, a tuning value) | 08-12 §2.4 (prior); 08-16 §2.1 |
| Porter travel | 15–30 km a travel day; walking 3–5 km/h (TCE 5 on a trail, 3 off) | 08-12 §2.4; 08-16 §2.1 |
| Transport in labour | 1.6 worker-days a tonne-km loaded, 3.2 with empty return (illustrative) | 08-12 §2.3 |
| Freight ratios | Sea 1 : river down 5 : river up 10 : wagon 52 (Roman); England sea 1 : river 5 : road 23 | 08-12 §2.2 |
| Return loads | Shipments toward busy markets about 14 % cheaper | 08-12 §2.2 |
| Loss exposure (test) | 0.5, 2, 10 % over 50 travel days; h = −ln(1 − P)/50 | 08-12 §2.4 |
| Financing (test) | 5, 15, 30 % a year | 08-12 §2.4 |
| Walking market catchment | 3.4–6.1 km to the remotest villagers | 08-05 §3.1 |
| Periodic markets | 2–3 sessions per 10 days; 2-, 4-, 8-day cycles; Aztec 5-day | 08-05 §3.1 |
| Ask change per review | 5–15 % (prior); TCE uses 5 % | 08-05 §3.2 |
| Seasonal gaps | Rice 16.6, maize 33.1, tomato 60.8 (units: see above) | 16-01 §2.2; 08-05 §3.1; 08-15 §2.5 |
| Post-harvest chain loss | 12.4–16.3 % (Rwanda); test 5–25 % | 08-16 §2.1, §2.2 |
| Disruption buffer; route cuts | 7–90 days; stress tests at 7, 30, 90 days | 08-16 §2.2 |
| Relative prices | Meat 3.19 kg bread; beef 1.74 kg rice; steak 5.38 × flour; flour 0.249 work-hours/kg (US 1901) | 08-15 §2.1 |
| Acceptance memory | 10–100 payments per half-life (prior; TCE uses 10) | 08-06 §2.3 |
| Money changing; mint charge | Spread 0.5–5 %; net mint charge 0–10 % (priors) | 08-06 §2.3 |
| Seigniorage | Usually under 2 % of English state revenue, 57 % in the Great Debasement | 08-06 §2.1 |

## 3. Validation

**Stylized facts: plausibility checks, not targets.**

| Pattern | Report | Test |
|---|---|---|
| Gaps fall to a band, not zero | 08-05 §1.7, §5 D; 08-12 §4 | Gaps of traded goods end near their carry band; goods inside the band never trade |
| Integration depends on movement cost | 08-05 §5 D | One save lived with the route cheap, dear (a longer walk) and blocked (an unfordable river): gaps order accordingly |
| Information works through behaviour | 08-12 §4; 08-05 §5 E | Reports passed at once against only by contact: fewer wasted trips and narrower gaps, with the same walking |
| Value per kg sets range | 08-12 §1.2, §4 | Tools' share of value traded rises with distance; grain's falls |
| Direction matters | 08-12 §4 | Bands and flows differ uphill and downhill |
| Trade buffers some shocks, transmits others | 08-12 §4; 08-16 §4 | Under one weather series both harvests fall together; a settlement short for its own reasons (soil, crowding) draws on the other |
| Availability is not access | 08-16 §1.7; 08-05 §5 H | A hungry settlement next to a fed one, its households holding nothing the sellers take, stays hungry, which is plausible |
| Entry and gluts | 08-12 §1.6, §4 | Many fetchers acting on one report produce a glut and losses, not a smooth path |
| Goods conserved in transit | 08-12 §5.5; 08-05 §6 | The ledger check covers loads; a carrier's death leaves the load somewhere |
| Correlation is not evidence | 08-12 §4; 08-05 §7 | Never graded |

**Dashboard row: price convergence (my proposal; every threshold a tuning value).**

- **Applies** to each pair of lived-in settlements in a world with at least 30 trades between them. Otherwise grey, with the reason (16-01 §4.5).
- **Gap.** Per good both offer in a month, 100 × |ln(median ask A ÷ median ask B)|. Asks are in hours of a capable adult's work, comparable across settlements. Realised prices are shown beside the asks (08-05 §6: an ask is not a transaction).
- **Band.** Per good, 100 × ln(1 + carry hours per unit ÷ price), from the walk hours inter-settlement trips actually spent on it, plus the controller's own 5 points.
- **Green:** for goods traded between the pair, the median monthly gap over the last ten years lies within the band, or is narrower than in the year before their first trade.
- **Amber:** gaps narrower than before contact but outside the band, or no traded good with 24 months of asks on both sides.
- **Red, for inconsistency only:** a good flowing on net from the dearer settlement to the cheaper over a year in which the gap exceeded the band.
- **Shown, not graded:** kilograms and units carried, trips, walk hours, outside buyers' share of each market, each settlement's money, the seasonal gap per settlement.
- Monthly gaps overlap and repeat, so they are not independent observations (16-01 §4.1, §4.5).

**The demo measure (my proposal).**
- Live one save from the month of first contact twice: once as it is, once with inter-settlement purchases switched off as a test harness, not a world rule. Where runs differ, live each three times, as the M3a demo did (plan §9).
- The demo shows the traded goods' gaps falling to their band in the first run and not in the second, with realised prices beside them. The twin is needed because both settlements share one weather series, and so co-move without trade.
- If the gaps do not close, the demo says so.
- The demo needs the settlements far enough apart for the walk to matter, at least a few kilometres. Next door, it shows nothing (08-05 §3.1's 3.4–6.1 km catchment).

**Does the weak seasonal gap matter for M5's convergence measure?** Indirectly, yes.

- **Not directly.** The convergence measure compares two settlements in the same month, so the level of seasonality does not enter it.
- **Grain is a weak test good.** Grain's cost anchor is the same world-wide, and one weather series drives both harvests. Grain asks in two settlements will start close and co-move. A green grain row could mean nothing, so the row should rest on goods whose own costs differ by place, read against the twin.
- **The same mechanics mute the signal.** Grain asks barely answer scarcity in time, for three reasons: unmet grain demand is never recorded (08-15 §5.5 and 08-05 §1.5 ask for funded attempts to count); the anchor carries storage but not expected scarcity (08-05 §1.5); and asks stay within about 5 points of the anchor. For the same reasons they will barely answer a neighbour's surplus or shortage. The signal that should draw grain to a hungry settlement is the same weak signal the food-prices row shows, seen across space instead of time.
- **Trade may lower the seasonal gap further, plausibly.** Reliable imports can smooth seasonal prices (08-05 §5 B), so the food-prices row's fixed 2 points may go red for a good reason once settlements trade. 16-01 §5 recommends a fitted seasonal component over several complete harvest cycles, and 08-15 §2.5 separates seasonal amplitude from shocks. Either is a better basis than a threshold.
- **Do not tune the controller to pass either row.** Plan §9 records that a failing run is not rerun until it passes. Fixing the missing scarcity terms is a model question for the designer, not a demo fix.

## 4. What to defer, and why

- **Coinage, mints, money changers and exchange rates.** Core content has nothing to mint, and money changers need currencies (08-06 §1.8). They come with metal content, whichever milestone adds it.
- **Credit, deferred settlement, partnerships and resident agents.** The plan puts credit in M7. These are the arrangements where trust and default matter (08-12 §1.3; 08-06 §1.2).
- **Tolls, tariffs, customs, smuggling.** These need officials, records, inspection and route control (08-12 §1.5). M7's demo is about tariffs.
- **Pack animals, carts and boats.** There is no livestock in content, and boats belong to M8's transport. Rivers are corridors with direction and season (08-12 §3), worth a design of their own.
- **Periodic markets, fairs and merchant associations** (08-05 §1.1; 08-12 §1.3). They belong in towns (M6), unless the demo needs one.
- **Haggling** (08-05 §1.3). 08-05 §6 finds one bargaining calculation enough for v1; the posted ask can stay as it is.
- **An outside world.** An unlimited export buyer becomes an unlimited source of money (08-12 §5.5). Never without explicit stocks.
- **A polity's store buying from its neighbour, and export bans** (08-16 §1.6; 08-05 §2). Both are cheap through the law pipeline, but each is a new move. See question 7.

## 5. Open questions for the designer

1. **Distance.** How far apart are the demo's settlements? Under about 2 km the markets merge; at 10–15 km a round trip takes most of a day; beyond that, people need nights away, which nothing does now.
2. **Metal.** Does M5 add metal content? If not, "per-polity currencies once minting exists" is deferred, and the demo omits it.
3. **The procurement anchor.** Does an ask's anchor become the lesser of own cost and replacement cost? For reports of other markets only, or for neighbours' offers too? The second changes every single-settlement world and needs a dashboard rerun.
4. **Scarcity in grain asks.** Should funded attempts to buy food be recorded as unmet demand (08-15 §5.5)? That would also move the seasonal row.
5. **Goods in transit.** Are they a ledger holder (an ADR), or does a purchase reach the buyer's stores at the seller's door as today? The difference is hours on M5's trips and days later.
6. **The food-prices row.** Keep the fixed 2 log points, move to a fitted seasonal component over complete cycles (16-01 §5), or grade it apart where a settlement imports grain?
7. **Polity moves.** Should a store buy from a neighbour in a famine, or a law forbid grain leaving, in M5? Both answer cases like Hazelstead's.
8. **Reports per household or per person.** Per household is cheaper; per person matches ADR-0016.
9. **The demo's twin.** Is a test switch that stops purchases between settlements acceptable, given that it must not change the world it is compared to?
