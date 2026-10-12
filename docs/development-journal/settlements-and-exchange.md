# Settlements, Movement and Exchange

This guide describes M5a settlement behavior and the implemented M5b exchange and diffusion systems in the integrated `main` tree as of 2026-10-10. [ADR-0018](../../decisions/0018-settlements-residence-movement.md) governs settlement identity and movement; [ADR-0019](../../decisions/0019-exchange-between-settlements.md) governs exchange. Inter-polity relations and works are documented separately in [Relations and Public Works](relations-and-public-works.md). Run measurements and known limits live in [PROJECT_PLAN.md §9](../../PROJECT_PLAN.md#9-decisions-log).

## Settlement identity and population accounts

A settlement has a permanent identity, a founding record, a parent when it was founded from another settlement, and its own polity. A household's current settlement determines its residence; being physically elsewhere on a trip does not change residence. Each person's residence history records settlement stays and off-map periods with dates and causes.

At world creation, the observer can add up to two neighboring founding groups. Candidate sites for all groups are drawn from a shared pool and assigned together, with a spacing constraint so their fields do not overlap. The group listing order therefore cannot select the best site. If the land cannot fit every requested group, the world records which group was left out. Whether the groups know one another's starting places is a setup choice.

Births, deaths, arrivals, departures and transfers between settlements update residence history. The annual settlement account is derived from those histories and checked against resident change:

```text
residents at year end = residents at year start
                     + births - deaths + arrivals - departures
```

Arrivals include their origin; departures include their destination where one exists. Leaving the simulated map is a real off-map account. The account is not independently saved, so history remains the authoritative record. Older supported saves reconstruct residence from the records they contain.

## Local contact and known places

Households hold a limited set of other settlements they know. Knowledge can come from being founded alongside another group, seeing its settlement while walking, hearing about it at a hearth, having kin there or visiting. A place record includes when it became known, who told the household and what a member last saw of its food. A household does not know every place merely because the world contains it.

This locality also applies to social institutions. Each polity, market, law history, watch, faction and gathering belongs to a settlement. People at a hearth associate there; they do not automatically share news or form ties across the map. Cross-settlement contact must arise through an implemented path such as a visit, kin, migration or a shared founding context.

Adults may visit a known, lived-in settlement within a day's walk. The visit is an activity with a trip there, time spent in company and a return trip. Its value comes from kin, ties and, for an unpartnered adult, a recent unsuccessful search for a partner at home; repeated visits fade in value. This gives people a reason to make a contact without scripting a route or meeting.

Marriage can form a household across settlements. The couple's new household settles where there is more usable land, then more room if the land is tied. Sex does not select the destination.

## Moving and founding

At a yearly review, a household compares its current place with places it knows. The evaluation weighs kin and people it regards there, food it has seen, grievances, expected harvest given up at home and the work of starting again. The same destination must remain preferred over two reviews before an ordinary move is carried out. Households do not receive an omniscient map or a globally ranked list.

Some departures have an immediate constraint. A household out of food that gives up goes to kin in another settlement when possible; an exile uses the same kin rule. If there is no such destination, the household leaves the map and is recorded off-map. Destination and cause remain inspectable in residence history and the chronicle.

The observer's migration-wave command sends a band of 5–50 related households from the nearest map edge over up to a week. It selects the amount of food they carry. Once they enter, they use the same household rules as everyone else; the command does not choose their lasting settlement or success.

Splinter founding is a household coalition project. A household weighs a site its people have walked, outside the field reach of existing settlements. It gathers kin and people its members regard; each invited household goes only if the coalition is preferable to its own best option and it can retain enough food and seed to reach its first harvest and two months beyond. The proposal must persist across two yearly reviews. New settlers found a polity using a copy of the body they left. A faction whose petition was rejected can also give its organizer's household a reason to consider leaving; faction members count among its social ties.

## Performance, checks and known gaps

The multi-settlement world shares exact-detail simulation. M5a profiling found work that grew with the whole world: global field scans, route searches and repeated company lookup. The implementation narrows many queries to a household or settlement, indexes fields by household, reuses exact route bounds and keeps per-settlement hearth lookup local. Tests compare optimized runs with digests where exact behavior is required; an equally fast route may differ if the prior route was not uniquely determined.

The measured full year for 3,000 people in three settlements was 1,179 seconds (19.6 minutes) at Max, versus 273 seconds for one settlement of 1,000. The design budget is ten minutes and remains unmet. An 8,000-person, three-group world was measured for thirty days, not established as a performance target. Gate B's current consistency fixtures contain one settlement, so they do not grade cross-settlement moves.

Annual resident accounts are checked against residence histories. In the post-routing thirty-year rerun of the M5a demo, 120 founders became 249 people; the accounts balanced, and the run recorded 63 household moves, including 48 from a temporary settlement founded by the migration wave. That settlement was abandoned within the year, and no coalition founded a settlement in this rerun. Earlier runs had different outcomes, so movement rates and founding are contingent observations, not targets. See [the plan's decision log](../../PROJECT_PLAN.md#9-decisions-log) for exact conditions.

## Trade by household reports

Households do not read another settlement's live market. A report records what a household believes a seller offers, in which payment goods, how much is available, when it learned the terms and whether a member saw them or a companion passed them along. Reports spread through ordinary contact and lose weight with age. A household can choose a reported offer only if the settlement is known and reachable within the authored `fetch` activity's walk and daylight limits.

### AP: buying by dated reports

One member travels to the seller. The seller's current offer and the buyer's holdings decide the exchange at the door, so an old report may lead to a recorded failed trip. Goods and payment move through the ordinary ledger at that point; goods are not modeled as a separate in-transit stock. Each trade is booked in the seller's market and records the buyer's settlement. The inspector exposes household reports, while the market panel summarizes outside buyers and recent exchange between settlements.

## Resale and price convergence

At a weekly review, a household can plan an errand to fetch a good it can spare and expects to sell at home. Quantity is bounded by reported stock, available means, carrying capacity and estimated home-market demand. The plan competes with other household activities; it is not an autonomous merchant agent. At the destination the household buys at current terms, carries goods home, and offers them through its existing market. Running errands through a firm is not implemented.

Households' replacement-cost anchors use their own price reports, payment-good valuations and estimated carrying work. Monthly convergence records compare asks and realized exchange by settlement pair. The dashboard uses a minimum of 30 purchases before grading a pair; it does not force prices toward a target. The observer may group same-day trips sharing a route into a caravan view, but a caravan is not a new actor.

## Diffusion through contact

People can learn a technique by watching relevant work or buying a good that only that technique makes. The knowledge record names the settlement from which it came, including when knowledge arrives with a person who moves. Households can also see finished buildings during contact; what they admire can influence later building taste, and a building records the one it followed. Provenance can therefore be traced across settlement boundaries. Taste and adoption remain household choices, not a command to copy.

## Evidence and limits

An earlier M5b comparison recorded 20 and 70 purchases between settlements over thirty years, with neither pair reaching the convergence row's 30-purchase threshold; realized price gaps remained within the measured carrying-cost band. Corrected river routing later separated the original neighboring demo villages. In the seed-9 rerun, they never met, so they did not trade or adopt each other's roof style; the 35 recorded purchases with trade were between Willowford and settlements its people founded. That rerun does not establish trade or diffusion between the original neighboring founders. A connected-world demo remains open, and these observations apply only to their recorded seeds and conditions.

The command `civ-host twin` can compare a saved world lived with cross-settlement buying enabled and with it stopped by a test harness. This is a diagnostic counterfactual, not a player-facing rule. Selected runs are not proof that trade or diffusion will happen in every world.

Per-polity currencies are not implemented because the content has no money-minting mechanism. The current system has no merchant firms that run errands, no goods in transit, and no guarantee of price convergence. See [PROJECT_PLAN.md §7](../../PROJECT_PLAN.md#7-milestones) and its [decision log](../../PROJECT_PLAN.md#9-decisions-log) for implementation details and measured outcomes.

## Further reading

- [ADR-0018: Settlement identity, residence and movement](../../decisions/0018-settlements-residence-movement.md)
- [ADR-0019: Trade between settlements](../../decisions/0019-exchange-between-settlements.md)
- [M5 development history](development-history.md)
