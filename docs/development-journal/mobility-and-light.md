# Mobility and Light

This guide covers implemented M10b systems in the integrated `main` tree as of 2026-10-24. M10b, rail and gas light, is closed. Its content and physical networks extend the same kernel-owned model used for roads, vehicles, work and household decisions. See [the M10 plan entry](../../PROJECT_PLAN.md#7-milestones) and [ADR-0034](../../decisions/0034-rail-and-gas-light.md). The transport model follows the research distinction between speed, useful load and reliable delivery ([research 07-08](../../research/07-technology/08-transport-tech.md)); the current rail implementation is an early, deliberately narrow system.

## Lamps and visibility after dark

Authored lamp vocabulary supports household-made tallow lamps and work by lamplight. Households that know the technique can make and use a lamp when their materials and expected cost permit; tallow is consumed and the lamp wears. Once light is present, the model changes what can be seen after dark and therefore what can be taken unseen. The clock reports homes lit and the observer can inspect work done by lamplight.

## Wagonways, rail and trains

A wooden wagonway is a built route with cells constrained by turns and ruling grade. Track is laid in stages using labor and timber; wagons can move material from pits toward home. Households can compare the cost of building a line and wagon against expected hauling savings. The observer draws incomplete and completed track and the positions of wagons.

A locomotive consumes coal for traction. Trains run along lines in blocks separated by passing loops, where they can wait for another movement. Household engines can bring extracted goods home. Lines expose stations at their ends; trains use timetables with up to four daily runs. Freight can be carried for a fee, and residents can ride to a pit when the train is quicker than walking, paying a fare in coal. A polity can also authorize a line as a public work and households contribute labor.

These are constrained early industrial transport mechanisms. The plan records a design nudge for selected engine, wagon, timetable and route choices. A nudge is an explicit bounded prior, not evidence that a feature emerges unaided or that the chosen values represent all historical railways.

## Gas networks and street lighting

A gasworks converts coal into gas and coke; gas is stored in a holder and delivered through mains. A main is laid by the works along permitted ways, one pipe section at a time. Connected buildings can burn gas in the evening when supply reaches them and the holder has enough; a building that cannot pay is disconnected until it can. Households pay the works in coal through the gas ledger channel. The observer shows main completion, a building's recent service or reason it was dark, and the works' holder and network.

A polity can authorize street lamps and name a lamplighter. The lamplighter makes a round after dark and burns tallow from the common store or their own stock. Light affects visibility along lit ways. The observer shows the lamps as they burn.

## Evidence and demo status

Rail, gas and lighting are implemented and represented in saved state and observer frames. The plan's fixtures and selected runs establish the stated code paths, not broad historical realism. There is no committed M10b observer video. The newest available video remains the M9 capture; the full M10 demo is planned for M10d. See [the demo gallery](../../demos/README.md).
