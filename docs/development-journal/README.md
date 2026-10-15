# Development Journal and Technical Guide

This folder records how TCE was built and explains the systems in the integrated `main` tree as of 2026-10-15. The source and project plan remain authoritative when later commits change implementation or status.

## Current baseline

M0–M3c provide the world, early farming society, economy, knowledge and buildings, weather and soils, and two simulation speeds. M4 adds settlement polities, law, cases, obligations, factions and political change. M5 adds multiple settlements, household trade and contact-based diffusion, agreements ratified by each polity's law, carried payments and log crossings. M6a adds daily water movement, wells, contamination, disease, household suspicion and response, a shared incidents view and a cholera demo. M6b adds fire and the response to it, floods, banks and ditches, rules for building, and a fire demo. M6c is underway: takings across the boundary, bands called and joined by choice, two-sided encounters by the minute, raids and bands told as claims, flight, and laws that send a band or forbid raiding are built; enclosures, sieges, terms and a polity over several settlements remain ahead. The Unreal client is still future work.

The current boundary versions are TCE wire 1.75, save schema 117 and content API 103. The M4 fifty-year dashboard has one failed food-price row in one world. M5a's measured year for 3,000 people in three settlements is about 19.6 minutes at Max, above the ten-minute design budget; Gate B does not yet grade cross-settlement movement. M6a's cholera demo ended with an outbreak through seven wells that people did not connect to those wells; a spring that infected nobody was suspected and closed. Fire escapes are now graded against a 1–100 per 1,000 household-years reference range; current fire-response findings remain limited to the recorded probes and tests. See [the plan's decision log](../../PROJECT_PLAN.md#9-decisions-log) for run conditions and interpretation.

## Read by question

| Document | Read this for |
| --- | --- |
| [Development history](development-history.md) | Milestones, implementation decisions, checks and observed outcomes through current M6 work |
| [Architecture and data flow](architecture-and-data-flow.md) | Kernel crates, ownership boundaries, content loading, observer and host communication |
| [Simulation model](simulation-model.md) | Time, landscape, households, economy, buildings and the high-level model for institutions and multiple settlements |
| [Institutions, law and social change](institutions-and-order.md) | Polities, deliberation, law, crime, grievances, factions, revolts and observer interventions |
| [Settlements and exchange](settlements-and-exchange.md) | Settlement identity, contact, movement, trade and diffusion |
| [Relations and public works](relations-and-public-works.md) | Claims, agreements, inter-polity payments and log crossings |
| [Water, sickness and fire](hazards-and-health.md) | Water sources and contamination, disease, incidents, fire spread and early response choices |
| [Persistence and interfaces](persistence-and-interfaces.md) | Content fingerprints, snapshots, wire frames, generated schemas and the C ABI |
| [Development and evidence practice](development-and-evidence.md) | CI, smoke worlds, dashboard and consistency gates, observed limits and reporting discipline |

## Keeping it current

When behavior or a boundary changes, update the README's short status, `PROJECT_PLAN.md`'s status and decision log, and the relevant guide here. Journal meaningful landed work with its goal, implementation, decision, checks, observed result and remaining uncertainty. Separate a design from an implemented feature, and a passing fixture from evidence of broad historical fidelity.

The project plan governs milestone scope and status. ADRs govern choices expensive to reverse. Rust, TypeScript, schemas and authored content define runtime behavior.
