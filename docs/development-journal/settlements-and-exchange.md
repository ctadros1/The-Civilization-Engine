# Settlements, Movement and Exchange

This guide describes implemented M5a behavior and the designed-but-not-implemented M5b boundary in public `main` at [`eb00281`](https://github.com/ctadros1/The-Civilization-Engine/commit/eb00281). [ADR-0018](../../decisions/0018-settlements-residence-movement.md) governs settlement identity and movement. [ADR-0019](../../decisions/0019-exchange-between-settlements.md) specifies the next trade milestone. Run measurements and known limits live in [PROJECT_PLAN.md §9](../../PROJECT_PLAN.md#9-decisions-log).

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

The M5a demonstration recorded annual resident accounts balancing and an average 4.3 moves between settlements per 100 residents per year for that run. Its migration wave's households later moved to a neighbor, and one coalition founded a settlement that survived a famine in that run. The result is evidence for those interactions in a selected demo, not a target rate or an expectation for every world. The journal records the demo's eventual departures and founding outcome as well.

## M5b design boundary

M5b is designed in [ADR-0019](../../decisions/0019-exchange-between-settlements.md), the [trade brief](../briefs/m5-trade.md) and the [diffusion brief](../briefs/m5-diffusion.md). Its slices are a design contract; at this baseline, the described behavior is not implemented.

### AP: buying by dated reports

A household would buy from a neighbor only when it has a current-enough price report, obtained by a member buying at that settlement or through a hearth contact and passed to others with ordinary routine news. Reports are household beliefs with an age; they are not a global market feed. A `fetch` activity would consider known settlements within a day's walk, obey daylight and travel constraints, and take the buyer to the seller. The seller's actual current offer would decide the trade at the door. If a report had gone stale, the failed attempt and reason would be recorded. Each trade would also record the buyer's settlement so it can be tallied locally.

### AQ: reselling and price convergence

Households would be able to weigh buying elsewhere and reselling at home against the work and risk of the trip; expected quantities are bounded by the depth of the home market. Repeated profitable activity could be handled through a household workshop. The household's ask would account for both its own cost and replacement cost known through reports. Monthly records by settlement pair would support a price-convergence dashboard row. A caravan would be an observer grouping of trips sharing route and day, not an autonomous group or a new market actor.

The demo is intended to compare the same saved world with cross-settlement purchases enabled and disabled by a host/test harness. The switch is not content vocabulary or a player-facing world rule. It isolates the contribution of trade while leaving each run's ordinary choices intact.

### AR: diffusion through contact

Technique and style diffusion would follow contact: a person can learn what they see or hear about while visiting, watching work or handling goods made with a technique. Provenance records which settlement a technique or building style came from; followed building designs can cross settlements. Household taste remains the chooser, while the founding way, accumulated taste and current building stock are tracked per settlement. No technology or roof style is forced to spread.

The planned neighbors row measures whether the designed contact paths produce cross-settlement diffusion; it is not a required target for every seed. If behavior remains absent after the planned time-box of tuning, the plan requires recording a `NUDGE:` rather than scripting adoption.

### AS: demo and explicit exclusions

The M5b demo would live the same save with and without the test harness's trade switch, three runs each. It would report realized prices and deviations from the design bands, and separately report style adoption using the diffusion brief's measure. A missing trade or style event is reported rather than manufactured.

Per-polity currencies are excluded until the content includes something that can mint money. M5b adds neither diplomacy nor infrastructure; treaties, tribute, bridges and enclosures belong to planned M5c. The source of truth for what has actually landed is the implementation status in [README](../../README.md) and [PROJECT_PLAN.md](../../PROJECT_PLAN.md#7-milestones).

## Further reading

- [ADR-0018: Settlement identity, residence and movement](../../decisions/0018-settlements-residence-movement.md)
- [ADR-0019: Trade between settlements](../../decisions/0019-exchange-between-settlements.md)
- [M5a development history](development-history.md)
