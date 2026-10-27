# Development Journal and Technical Guide

This folder records how TCE was built and explains the systems in the integrated `main` tree as of 2026-10-27. The source and [project plan](../../PROJECT_PLAN.md) remain authoritative when commits change implementation or status.

## Current baseline

M0–M9 are implemented. M10a (power and the works), M10b (rail and gas light) and M10c (services and letters) are implemented. M10d is underway: household money valuation, custody, redeemable notes, lending, runs, suspension and bearer-heard claims are implemented in slices EF–EI. Polity-issued notes, counterfeiting and the complete M10 demo remain planned. The Unreal client and modern civilization remain future work.

Current boundary versions are TCE wire 1.106, save schema 214 and content API 195. The newest committed observer video is M9. No M10 milestone video is committed; the complete M10 demo remains planned for M10d. Run reports are seed-specific evidence, not proof of typical outcomes or historical realism.

## Read by question

| Document | Read this for |
| --- | --- |
| [Development history](development-history.md) | Milestones, implementation decisions, checks and observed outcomes through current M10c work |
| [Architecture and data flow](architecture-and-data-flow.md) | Kernel crates, ownership boundaries, content loading, observer and host communication |
| [Simulation model](simulation-model.md) | Time, landscape, households, economy, buildings and the high-level model for institutions and multiple settlements |
| [Institutions, law and social change](institutions-and-order.md) | Polities, deliberation, law, crime, grievances, factions, revolts and observer interventions |
| [Settlements and exchange](settlements-and-exchange.md) | Settlement identity, contact, movement, trade and diffusion |
| [Relations and public works](relations-and-public-works.md) | Claims, agreements, inter-polity payments, log crossings, war and conquest |
| [Water, sickness and fire](hazards-and-health.md) | Water sources and contamination, disease, incidents, fire spread and early response choices |
| [Persistence and interfaces](persistence-and-interfaces.md) | Content fingerprints, snapshots, wire frames, generated schemas and the C ABI |
| [Industry and power](industry-and-power.md) | M10a's power, fuel, iron and works systems |
| [Mobility and light](mobility-and-light.md) | M10b lamps, wagonways, trains, freight, gas mains and street lighting |
| [Services and care](services-and-care.md) | M10c care beyond the household and birth-care systems |
| [Money, custody and notes](money-and-notes.md) | M10d household valuation, safekeeping, redeemable notes, lending, runs and claims |
| [Development and evidence practice](development-and-evidence.md) | CI, smoke worlds, dashboard and consistency gates, observed limits and reporting discipline |

## Keeping it current

When behavior or a boundary changes, update the README's short status, `PROJECT_PLAN.md`'s status and decision log, and the relevant guide here. Journal meaningful landed work with its goal, implementation, decision, checks, observed result and remaining uncertainty. Separate a design from an implemented feature, and a passing fixture from evidence of broad historical fidelity.

The project plan governs milestone scope and status. ADRs govern choices expensive to reverse. Rust, TypeScript, schemas and authored content define runtime behavior.
