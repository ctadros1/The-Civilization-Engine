# Proposal (status: **Proposed**, not decided): a sewer rate paid by connected homes

Backlog C's "Not yet" list (plan §7, M11f's sewers) leaves open "a sewer rate paid by connected
homes (it would need a channel)". This brief turns that line into a design the owner can accept,
amend or reject. Nothing here is implemented, and it records no decision.

## What exists now (claude/claude-development 48cd61b4)

- **Sewers** (ADR-0041; `civ-land/src/sewers.rs`, `civ-agents/src/population/sewers.rs`): a polity
  lays one by law (`lay_sewer`), or a household does. Homes along it connect their privies, and
  each day's decaying share goes into the sewer in connection order while capacity lasts. A broken
  segment heaps its connections' share again until it is mended. Laying, clearing and relining are
  done by the keeper's household, using material from its own store, "paid where a law of pay for
  common work is in force" (`sewer_work`). The keeper is a household, or the polity's store (`None`).
- **No user charge.** A connected home pays nothing. Its keeper, or the polity's common store
  through pay for common work, bears all the upkeep.
- **Precedents for a charge:** gas sold by the meter on `Channel::Gas` with cut-off (M10b slice
  DY); current on `Channel::Wire` (M11a); refuse on `Channel::Refuse`. The ledger channels end at
  `Refuse = 36`.

## What the research says

- 12-08 §1 (opening; §1.1): treat infrastructure as "a continuing relationship between an asset,
  its users, and an organization — not as a construction purchase followed by a fixed upkeep
  charge". User associations, municipal revenue, concessions and endowments are all historical
  arrangements. None of them is mandatory.
- 12-08 §1.2: support for an arrangement weighs expected benefit and norms against contribution
  burden and expected unfairness. Contribution and allocation are separate problems.
- 12-08 §1.4 and §4.3: maintenance needs an organization of its own, and routine care does not
  substitute for renewal.
- **The reports give no sewer rate.** Any figure below is a design prior or is derived from the
  keeper's actual costs, as gas and wire prices were.

## Proposed design

1. **A law, not a default.** A polity may set `sewer_rate` (a `PolicyKind` appended after the
   existing kinds) on a sewer it keeps. It is proposed only where its store paid for that sewer's
   upkeep in the last year: an issue raised when the common store spent on a sewer it keeps. This
   keeps the vocabulary, never plot, rule: a world with no sewer, or no upkeep paid, never sees the
   option.
2. **The rate comes from cost, not tuning.** At the polity's yearly review, the rate per connected
   home is the store's spending on that sewer over the past year (material at the store's own cost
   in hours, plus hours paid under pay for common work), divided by the homes connected. It is paid
   in the store's money or in kind, as the levy is (M10d EF: dues to the store in its money). There
   is no new tuning value.
3. **A new channel**, `Channel::SewerRate = 37` (appended; saves bump). It is collected monthly at
   the same point the levy is collected, so it adds no new scheduler event.
4. **Cut-off as for gas.** A home that cannot pay keeps its connection, but its privy heaps again
   (the connection's `served` is false, with cause `CutOff`) until it pays. Disease loads follow the
   heap exactly as they do for an unconnected home. There is no fine and no extra penalty.
5. **Weighing the law.** Households weigh it as they weigh the levy: connected households weigh
   the rate against the hours the sewer spares them (`sewer_spared_h`, which already exists);
   unconnected ones weigh the store's spending that it relieves. A household-run sewer gets no law.
   Its keeper already bears its costs and may set terms only if a later brief adds posted terms.
6. **Observer.** The sewer's page shows the rate in force, homes paying, homes cut off, and last
   year's upkeep spent.

## Acceptance examples (tests to write)

- **No sewer, no option:** a world without sewers never raises the issue or lists the law.
  Section digests are unchanged except for the length-only effect of the new channel, read as
  M10's slices read it: every report line is unchanged.
- **Rate from cost:** a polity sewer with 4 connected homes and 120 hours of upkeep paid by the
  store last year gives a rate of 30 hours a home per year, charged monthly as 2.5 hours' worth in
  the store's money.
- **Cut-off:** a connected home holding nothing to pay heaps its privy share from the next day,
  with cause `CutOff`. Once it pays it is served again, and the order of connections is unchanged.
- **The law's weighing:** with the rate below the hours the sewer spares them, connected
  households support it. With the rate above, they oppose it. Unconnected households' support
  rises with the store's upkeep spending.
- **Saves:** the law, the rate and the cut-off flags save and load exactly (save, load, save,
  equal bytes).
- **Long run:** the 50-year dashboard is unchanged in rows that sewers do not touch. No threshold
  moves.

## Bounded implementation plan

Four commits, each with its own tests:
1. Channel, issue and law kind, with no effect.
2. Rate computed at the polity's review, and collection.
3. Cut-off through the existing served/cause path.
4. Observer words.

Estimate: about two working sessions. It touches `ledger.rs`, `polity.rs` (issue and law),
`population/sewers.rs`, `persist/agents.rs` (save bump) and a sewer test file. The wire bump
applies only if the observer step is included.

## Uncertainties for the owner

- Whether a polity should also be able to charge for an unconnected home's benefit (cleaner wells
  downstream), as a general levy already does. This proposal leaves that to the levy.
- Whether the store's own hours should count at its cost, or only at paid hours. The proposal
  counts both, as gas prices count the works' costs.
- Monthly collection follows the levy. The research gives no cadence.
