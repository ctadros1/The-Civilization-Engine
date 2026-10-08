# Architecture decision records

ADRs exist only for decisions that are **expensive to reverse**: schema, persistence, and the
FFI boundary ([plan §1, rule 7](../PROJECT_PLAN.md#non-negotiable-rules)). Everything else is a
one-line entry in the plan's decisions log (§9). If a milestone needs more than about three ADRs,
the milestone is too big and gets split.

| ADR | Title | Status | Milestone |
|---|---|---|---|
| [0001](0001-boundary-schema.md) | Boundary schema: `commons-wire` envelope with FlatBuffers payloads | Accepted | M0 |
| [0002](0002-snapshot-container.md) | Snapshots: `commons-persist` chunked container with FlatBuffers sections | Accepted | M0 |
| [0003](0003-people-movement-history.md) | People, movement and history in the kernel, saves and boundary | Accepted | M1 |
| [0004](0004-buildings-land-paths.md) | Building specs, land state and paths | Accepted | M1 |
| [0005](0005-kernel-c-interface.md) | The kernel's C interface | Accepted | M2 |
| [0006](0006-goods-ledger-firms.md) | Goods, the ledger, prices and firms | Accepted | M3a |
| [0007](0007-claims-property-regimes.md) | Claims and property regimes | Accepted | M3a |
| [0008](0008-knowledge-carried-by-people.md) | Knowledge carried by people | Accepted | M3b |
| [0009](0009-building-components.md) | Building components: grammar v2, condition and trust | Accepted | M3b |
| [0010](0010-ground-people-change.md) | The ground people change: deposits and earthworks | Accepted | M3b |
| [0011](0011-execution-modes.md) | Execution modes: Detailed and Accelerated | Accepted | M3c |
| [0012](0012-weather-and-soil.md) | Weather and the soil | Accepted | M3c |
| [0013](0013-polity-offices-laws.md) | The polity, its offices and its laws | Accepted | M4a |
| [0014](0014-ties-standing-notables.md) | Ties, standing and notables | Accepted | M4a |
| [0015](0015-incidents-cases-obligations.md) | Incidents, cases and obligations | Accepted | M4b |
| [0016](0016-grievances-claims-opinions.md) | Grievances, claims and opinions, and the interventions that touch them | Accepted | M4c |
| [0017](0017-factions-episodes-regime-change.md) | Factions, episodes and changes of regime | Accepted | M4c |

## Template

```markdown
# ADR-NNNN: Title

Status: Proposed | Accepted | Superseded by ADR-XXXX
Date: YYYY-MM-DD
Milestone: Mn

## Context
What forces the decision; which research reports were read.

## Decision
What we do, precisely enough that code review can check it.

## Consequences
What gets easier, what gets harder, what is now forbidden.

## Alternatives considered
Each with the reason it lost.

## Revisit when
The concrete condition that would reopen this.
```
