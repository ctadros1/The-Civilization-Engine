# Development Journal and Technical Guide

This folder records how TCE was built and explains the systems in the current repository. The public `main` baseline is commit [`eb00281`](https://github.com/ctadros1/The-Civilization-Engine/commit/eb00281), merged 2026-10-09. It includes M0–M4, M5a and the M5b design. The source and project plan remain authoritative when later commits change implementation or status.

## Current baseline

M0–M3c provide the world, early farming society, economy, knowledge and buildings, weather and soils, and two simulation speeds. M4 adds settlement polities, law, cases, obligations, factions and political change. M5a adds multiple settlements, contact, visits, marriage, migration and coalition founding. M5b is designed but its cross-settlement trade and diffusion mechanisms are not yet implemented. The Unreal client is still future work.

The current boundary versions are TCE wire 1.53, save schema 55 and content API 55. The M4 fifty-year dashboard has one failed food-price row in one world. M5a's measured year for 3,000 people in three settlements is about 19.6 minutes at Max, above the ten-minute design budget; Gate B does not yet grade moves between settlements. See [the plan's decision log](../../PROJECT_PLAN.md#9-decisions-log) for run conditions and interpretation.

## Read by question

| Document | Read this for |
| --- | --- |
| [Development history](development-history.md) | Milestones, implementation decisions, checks and observed outcomes from the initial plan through M5a and the M5b design |
| [Architecture and data flow](architecture-and-data-flow.md) | Kernel crates, ownership boundaries, content loading, observer and host communication |
| [Simulation model](simulation-model.md) | Time, landscape, households, economy, buildings and the high-level model for institutions and multiple settlements |
| [Institutions, law and social change](institutions-and-order.md) | Polities, deliberation, law, crime, grievances, factions, revolts and observer interventions |
| [Settlements and exchange](settlements-and-exchange.md) | Settlement identity, contact, visits, migration, founding, and the designed M5b trade/diffusion boundary |
| [Persistence and interfaces](persistence-and-interfaces.md) | Content fingerprints, snapshots, wire frames, generated schemas and the C ABI |
| [Development and evidence practice](development-and-evidence.md) | CI, smoke worlds, dashboard and consistency gates, observed limits and reporting discipline |

## Keeping it current

When behavior or a boundary changes, update the README's short status, `PROJECT_PLAN.md`'s status and decision log, and the relevant guide here. Journal meaningful landed work with its goal, implementation, decision, checks, observed result and remaining uncertainty. Separate a design from an implemented feature, and a passing fixture from evidence of broad historical fidelity.

The project plan governs milestone scope and status. ADRs govern choices expensive to reverse. Rust, TypeScript, schemas and authored content define runtime behavior.
