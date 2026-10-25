# Development Journal and Technical Guide

This folder records how TCE was built and explains the systems in the integrated `main` tree as of 2026-10-22. The source and project plan remain authoritative when later commits change implementation or status.

## Current baseline

M0–M9 are implemented. M8 adds city-scale transport, land planning and structural behavior; M9 adds household resolution for settlements away from the observer's focus, time skips, climate changes and a ten-settlement lived run. The current milestone is M10a: river-powered installations, powered recipes and household-built water mills are implemented; fuel and iron works continue. The Unreal client and modern civilization remain future work.

Current boundary versions are TCE wire 1.96, save schema 174 and content API 160. The newest committed observer video is M9; M9's ten-year run report remains the full simulation record, while the gallery video is a shorter presentation capture. M10a is underway and has no milestone video yet. Recorded outcomes are seed-specific. See [the plan's decision log](../../PROJECT_PLAN.md#9-decisions-log) for run conditions, failures and interpretation.

## Read by question

| Document | Read this for |
| --- | --- |
| [Development history](development-history.md) | Milestones, implementation decisions, checks and observed outcomes through current M10a work |
| [Architecture and data flow](architecture-and-data-flow.md) | Kernel crates, ownership boundaries, content loading, observer and host communication |
| [Simulation model](simulation-model.md) | Time, landscape, households, economy, buildings and the high-level model for institutions and multiple settlements |
| [Institutions, law and social change](institutions-and-order.md) | Polities, deliberation, law, crime, grievances, factions, revolts and observer interventions |
| [Settlements and exchange](settlements-and-exchange.md) | Settlement identity, contact, movement, trade and diffusion |
| [Relations and public works](relations-and-public-works.md) | Claims, agreements, inter-polity payments, log crossings, war and conquest |
| [Water, sickness and fire](hazards-and-health.md) | Water sources and contamination, disease, incidents, fire spread and early response choices |
| [Persistence and interfaces](persistence-and-interfaces.md) | Content fingerprints, snapshots, wire frames, generated schemas and the C ABI |
| [Industry and water power](industry-and-power.md) | M10a's saved power installations, river-derived output, powered recipes and household mill choice |
| [Development and evidence practice](development-and-evidence.md) | CI, smoke worlds, dashboard and consistency gates, observed limits and reporting discipline |

## Keeping it current

When behavior or a boundary changes, update the README's short status, `PROJECT_PLAN.md`'s status and decision log, and the relevant guide here. Journal meaningful landed work with its goal, implementation, decision, checks, observed result and remaining uncertainty. Separate a design from an implemented feature, and a passing fixture from evidence of broad historical fidelity.

The project plan governs milestone scope and status. ADRs govern choices expensive to reverse. Rust, TypeScript, schemas and authored content define runtime behavior.
