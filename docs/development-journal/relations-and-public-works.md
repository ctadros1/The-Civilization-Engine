# Relations and Public Works

This guide describes implemented M5c systems in the integrated `main` tree as of 2026-10-10. [ADR-0020](../../decisions/0020-relations-between-polities.md) governs inter-polity relations and agreements; crossing construction extends [ADR-0004](../../decisions/0004-buildings-land-paths.md) and [ADR-0009](../../decisions/0009-building-components.md). The plan's [M5c milestone](../../PROJECT_PLAN.md#7-milestones) and [decision log](../../PROJECT_PLAN.md#9-decisions-log) hold exact slice details, checks and run results.

## What a polity knows about another

Each settlement has its own polity and laws. Relations are not stored as a score or a world-level diplomatic mode. People hold views based on acts they saw or heard about; grievances name a responsible party and an expectation; claims and agreement histories record what the polities did. A pure classifier derives labels such as *unknown*, *known*, *wary*, *friendly*, *under agreement* or *tributary*. Labels are explanatory views only and do not drive simulation choices. The two polities may have different labels because their people have different knowledge and experience.

Claims over wild gathering places and deposits begin as laws decided by the local polity. Households record where their people gathered or dug and whether they saw outsiders working there. When a household knows its own polity's claim and its people work beside outsiders at that place, a grievance can form. Claims and news about them travel through people and routine contact; there is no global diplomatic feed. The system does not yet model raids or taking goods from another settlement's stores.

## Agreements: one record, two laws

An agreement is a shared record between two polities, while its authorization is kept in each polity's own law history. Negotiators consider bounded packages using their own household forecasts and anticipated support. Each side must sponsor and ratify the terms under its own custom. The agreement takes effect only after both sides have passed it and each has heard the other's decision through a traveler.

Current clauses cover leave to use a claimed place, a one-time gift, and recurring transfers. A payment is drawn from a polity's common store, set aside in its own ledger holder, and carried by a person to the receiving settlement. The records distinguish what was paid in, set aside, owed and received. A payment that misses its delivery window records why; evidence of performance reaches people by contact and can affect their views. Agreements do not create goods, and a settlement without the required store cannot pay from nowhere.

The agreement and law views expose the terms, status and the two sides' histories. A test world demonstrates one gathering rejecting terms its negotiator expected to pass. In the recorded thirty-year M5 demo, no agreement reached a gathering, so that failure has not yet been observed in a lived demo. Shared claims leave little reason to negotiate leave when both villages use the same wild place; the documented alternative of withholding contested places was not adopted because it destabilized the probe village. The implementation is kept plausible and the unresolved funnel is reported rather than hidden.

## Crossings over water

A bridge system is authored content; the current system is a log footbridge. A crossing stores its site, spans, members, owner, work, state and condition. The bridge's deck becomes walkable when open, and routing is recalculated when it opens or fails. Loads are checked as people step on and during daily updates; timber decays with exposure. A failed crossing drops a person on it and closes the route.

A household can consider a crossing using its recorded river wades and the walking time the deck could save. People in the household choose construction work, with quality based on their building skill. A polity can also pass a public-work law for a crossing where the households' combined use makes the work worthwhile; each household is asked for an equal share and contributes by choice. A bridge system's crew limit bounds how many workers can make progress on it in a day. The observer's crossing view shows construction, owner, members, condition and location on the map.

In the measured worlds, no household's own wades justified a standalone log bridge. Selected village runs did build shared crossings. The current system does not include a trestle over the trunk river, other bridge systems, routes made reachable only by a crossing, abandoning stalled work or repairing a bridge before it fails. Enclosures were moved to M6, where theft and force can give them a purpose.

## Source map

| Concern | Implementation |
| --- | --- |
| Household views and place use | `kernel/crates/civ-agents/src/views.rs`, `uses.rs`, `population/relations.rs` |
| Agreements and clauses | `kernel/crates/civ-agents/src/agreements.rs`, with lifecycle updates in `population/relations.rs` |
| Derived relation labels | `kernel/crates/civ-sim/src/relations.rs` |
| Crossing model and daily work | `kernel/crates/civ-agents/src/bridge.rs`, `population/crossings.rs` |
| Observer views | `kernel/crates/civ-sim/src/frames/government.rs`, `frames/crossings.rs` |
| Integration evidence | `kernel/crates/civ-sim/tests/relations.rs`, `agreements.rs`, `crossings.rs` |

## Further reading

- [ADR-0020: Relations between polities](../../decisions/0020-relations-between-polities.md)
- [M5 settlement, trade and diffusion guide](settlements-and-exchange.md)
- [M5 development history](development-history.md)
- [Project plan and exact evidence](../../PROJECT_PLAN.md#7-milestones)
