# ADR-0027: Claims and coin

Status: Accepted
Date: 2026-10-16
Milestone: M7d

## Context

Through M7c a household gets what it lacks by its own work, by asking for help (a gift, recorded
in the tie's help balance and in `Provision` evidence), by buying at a door at posted asks in
hours (ADR-0006), or by taking. Obligations exist only as ADR-0015's record of what a finding or a
demand imposes, in kcal. A market's money is whatever good its trades settle in most (slice I's
inferred commodity money), valued by each seller at its own cost of getting it. There is no loan,
no deposit, no metal and no coin. Plan §7's M7 asks for "credit and banking" and "coin debasement
and inflation", and plan §5.3 for money that emerges from barter and is trusted or not.

The money brief (`docs/briefs/m7-money.md`) reads 08-05, 08-06, 08-07, 08-10, 08-11, 08-13, 08-20
and 06-10. Its invariants:

- help and debt are two records: need-based transfers create no repayment debt (08-11 §1.1);
- early credit is seasonal advances in goods between households, known to witnesses, not writing
  (08-07 §3.1; 08-06 §5.5);
- a lender decides from beliefs: i_min = (cT + F/P + p(1 − R)) / ((1 − p)T) (08-07 §1.3);
- default has answers (standstill, extension, haircut, seizure), and need not end lending
  (08-07 §1.8); bondage is not inferred from debt (08-10 §1.1);
- a bank is what a holder does, not a kind of thing (08-07 §1.6); reserves are not capital;
- a coin is worth to its holder what it buys, and the quantity of money reaches prices only
  through holders' beliefs (08-06 §1.3, §1.10);
- debasement changes the next issue, never coins already struck; Gresham follows from who can
  tell cohorts apart (08-06 §1.5, §1.7).

## Decision

### 1. Claims in goods, a book of their own

- A *claim* is a saved record apart from ADR-0015's obligations (whose meaning, kcal imposed by a
  finding, stays): creditor and debtor households, the good lent and the amount, the good owed and
  the amount, the day made and the day due, its pledge (none, goods set aside under the
  creditor's roof, a field's use, or a guarantor household), its standing (open, repaid, extended,
  defaulted, forgiven), payments made, and its witnesses (who saw it made).
- Two appended ledger channels, `Lent` and `Repaid`, so the conservation check covers both legs.
  Gifts stay gifts: the help path is untouched.
- Claims are sparse and due on dates: a maturity queue, not a daily scan.

### 2. Asking and lending

- A household asks when its plan to the next harvest falls short after help: seed before sowing,
  food beyond what help gave, a tool. The ask names what it is for and its due day (the harvest).
- Candidate lenders are households and workshops its members hold ties to, then any holding more
  than they keep (the gift rule's test), within the trade walk; at most a few are asked.
- The lender decides by 08-07 §1.3 from beliefs: p from its ties' `Word` evidence of the asker's
  household and the harvest it expects for it, R from the pledge, c what the good would earn or
  lose kept at home. It asks the larger of i_min and the content's customary increment, and
  refuses above a believed p. Interest is measured (realised returns after losses), never set.

### 3. Due day, default and pledges

- At the due day the debtor pays, pays part, asks for time or refuses, weighed as a finding is
  met today. The creditor grants time, takes the pledge (goods it holds; a field's use on the
  existing lease; a guarantor's payment) or demands, as for takings.
- A missed payment writes bad `Word` evidence for the debtor's household in the creditor's ties,
  told by word.
- An appended policy kind, *debts heard*, lets the gathering or a forum hear a creditor's case;
  its bundle is the custom's choice among an extension in a lean year, the pledge taken, or
  labour-days owed to the creditor (ADR-0015 §5's kind). Finite labour-days are the harshest
  bundle the core offers; no unfree status is authored.
- A workshop that cannot pay a wage owes it as a claim, and fails only when unpaid by its due day.

### 4. Keepers and the store

- A household with roofed room may keep others' grain for a fee in grain on an appended `Custody`
  channel; the grain is the depositor's, never counted as the keeper's own.
- A keeper may lend deposits by its own choice; it then owes more on demand than it holds, and a
  run is depositors' choices (hearing the keeper lent, or a debtor of the keeper defaulted, or
  seeing others withdraw), paid in a seeded first-come order.
- An appended policy kind, *lend from the store*: seed or food lent from the common store at the
  law's terms, repaid into it, carried by the keeper. Relief stays a gift.
- "Bank" and "moneylender" are derived labels.

### 5. Money's worth to its holder (coin and weighed metal only)

- Once a market settles in coin or weighed metal, each household keeps a dated belief, in its own
  hours, of what a unit bought in its recent purchases, fading by the market's `memory_days`. It
  values a unit at the greater of its use value and that belief, less what verifying it costs;
  beyond a desired balance (7–30 days of its money spending, a prior) a unit is worth less by the
  stores rule, target ÷ (target + held). This is the only route from the quantity of money to
  prices. Inferred commodity money (grain, firewood) keeps its own-cost valuation, so a world
  without metal gives the build-before's digests.

### 6. Metal, mint and debasement

- Metal is content: an ore at deposits, a smelting technique and a metal good, discovered and
  spread as techniques are (ADR-0008); it may be paid by weight before any coin.
- A mint is a recipe (metal and fuel in, coins of a cohort out) gated by a technique and by a
  polity law naming an office to mint. A cohort is saved (issuer, day, mass, fineness, a mark);
  purses hold counts by cohort.
- The issue enters by transactions: households bring metal for coin less the law's fee; the
  polity pays coin for shares of its public works (the first polity spending). A levy may name
  coin as its medium.
- Debasement is a mint law setting the next cohort's mass, fineness or fee. Who can assay (those
  who know smelting) values coin by metal; others by count until their purchases teach otherwise;
  a payer gives the cohort worth least to them.
- Foreign coin is valued by weight at a discount, less assay, until used; that discount is the
  emerging exchange rate, derived by the observer.

### 7. Guards

- No date, era, population or wealth opens credit or coin; no issue weighs toward a loan or a
  mint; no interest by era; no inflation coefficient; no index read by behaviour; no default die;
  no rescue that creates goods.
- A world with no loan, deposit or metal gives the build-before's digests (tested per slice).
- If nobody lends after a time box, the demo nudges a prior (the customary increment, the trust
  `Word` evidence gives), never a loan, logged as `NUDGE:`.

## Slices

- **CA**: loans in goods between households: the claim book, asking and lending by i_min from
  ties, harvest due days, repayment, `Word` evidence from what was repaid, the observer's claims.
- **CB**: default and pledges: answers at the due day, goods, field use and guarantors as pledges,
  *debts heard* with finite labour-days, wage arrears.
- **CC**: keepers and the store: custody for a fee, lending deposits with runs possible, *lend from
  the store*.
- **CD**: money's worth to a holder, for weighed metal and coin, checked against today's markets.
- **CE**: metal, mint and debasement: the metal content, the mint recipe and law, cohorts and
  purses, polity spending and a levy in coin, foreign coin by weight; the M7d demo (time-boxed,
  NUDGEs logged).

## Consequences

- Easier: lean years can be bridged, or deepened, by people's choices; the M7 demo's debasement
  becomes reachable where metal is found.
- Harder: a second obligation record beside ADR-0015's; ledger channels gain `Lent`, `Repaid` and
  `Custody`; purses add a holding kind.
- Without metal in a world, there is no coin and no debasement: the demo says so.

## Alternatives considered

- **Generalise ADR-0015's obligation from kcal to goods.** Lost: findings, demands and their
  saves would all change meaning; a separate book keeps both exact.
- **Coin before credit (the plan's ladder).** Lost: the reports put credit in goods first, and coin
  needs metal the content does not have yet.
- **Money's worth by belief for every inferred money.** Lost for now: it changes every world's
  prices and needs a dashboard rerun (brief open question 7).
- **Circulating receipts.** Deferred: assignment and acceptance rules (08-06 §5.5).

## Revisit when

- Receipts or bills circulate between settlements (assignment, acceptance).
- Polities borrow (public debt) or default.
- Land is sold (mortgages).
