# Development Journal and Technical Guide

This folder explains how TCE has been built, how the integrated simulation works, and where its main code boundaries are. These notes are reconciled through `main` commit [`33dbd4a`](https://github.com/ctadros1/The-Civilization-Engine/commit/33dbd4a), dated 2026-10-05, which records M3c slice U complete. They complement the milestone plan, architecture decision records (ADRs), code comments, and research library; they do not replace them.

## Source baseline

The technical baseline is the public `main` tree at commit [`33dbd4a`](https://github.com/ctadros1/The-Civilization-Engine/commit/33dbd4a). M0–M3b are implemented; M3c slices S–U are integrated, and V (soils and fertility) and W (Accelerated-mode approximations, tuning and the demo) remain. The project plan §9 contains the detailed parameters, checks and observed outcomes. Later commits may change implementation or status, so consult the plan and source before relying on a moving detail.

## Read by question

| Document | Read this for |
| --- | --- |
| [Development history](development-history.md) | How the project moved from plan to a working village simulation, milestone by milestone |
| [Architecture and data flow](architecture-and-data-flow.md) | Processes, crates, authority boundaries, content loading, commands, queries and observers |
| [Simulation model](simulation-model.md) | Time, terrain, people, decisions, households, economy, buildings and knowledge |
| [Persistence and interfaces](persistence-and-interfaces.md) | Content fingerprints, saves, wire frames, generated schemas and the C ABI |
| [Development and evidence practice](development-and-evidence.md) | Milestone workflow, CI, smoke worlds, results, limitations and how to extend this journal |

## Keeping it current

When implementation materially changes, update the short status in the root [README](../../README.md), the [plan](../../PROJECT_PLAN.md), and the relevant page here. Journal an actual development step after it lands: record its intent, source changes, decision or trade-off, checks run, observed result, and remaining uncertainty. Do not describe a proposed feature as implemented or a passing fixture as proof of real-world fidelity.

The plan remains authoritative for scope and milestone decisions. ADRs remain authoritative for decisions expensive to reverse. The Rust and TypeScript source remains authoritative for runtime behavior.
