# Architecture decision records

ADRs exist only for decisions that are **expensive to reverse**: schema, persistence, and the
FFI boundary ([plan §1, rule 7](../PROJECT_PLAN.md#non-negotiable-rules)). Everything else is a
one-line entry in the plan's decisions log (§9). If a milestone needs more than about three ADRs,
the milestone is too big and gets split.

| ADR | Title | Status | Milestone |
|---|---|---|---|
| [0001](0001-boundary-schema.md) | Boundary schema: `commons-wire` envelope with FlatBuffers payloads | Accepted | M0 |
| [0002](0002-snapshot-container.md) | Snapshots: `commons-persist` chunked container with FlatBuffers sections | Accepted | M0 |

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
