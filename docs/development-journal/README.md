# Development Journal and Technical Guide

This folder explains how TCE has been built, how its current simulation works, and where the main code boundaries are. It complements the milestone plan, architecture decision records (ADRs), code comments, and research library; it does not replace them.

## Source baseline

These pages describe the development branch `claude/adoring-turing-w10ius` as it stood on 2026-10-07, with slice V's first two steps in place (each field's soil and the record of its harvests, and households planning from those records, saves schema 26, content API 28); its history names each step's commit. At that point M3b (knowledge and building) is implemented, and so are M3c slices S (the speeds and the day step), T (the fifty-year dashboard) and U (weather and seasons); slice V (soils) is in progress and slice W (tuning and the demo) remains. The journal was first written against the public `main` tree at [`fad23fa`](https://github.com/ctadros1/The-Civilization-Engine/commit/fad23fa) (2026-10-04), when slice Q had only its deposit-placement primitive; the [development history](development-history.md) records what has landed since. Treat claims about implementation as belonging to this source baseline. Later commits may change the code or milestone status; check the current source and plan before relying on a detail.

## Read by question

| Document | Read this for |
| --- | --- |
| [Development history](development-history.md) | How the project moved from plan to a working village simulation, milestone by milestone |
| [Architecture and data flow](architecture-and-data-flow.md) | Processes, crates, authority boundaries, content loading, commands, queries and observers |
| [Simulation model](simulation-model.md) | Time and speeds, terrain, weather, people, decisions, households, economy, buildings and knowledge |
| [Persistence and interfaces](persistence-and-interfaces.md) | Content fingerprints, saves, wire frames, generated schemas and the C ABI |
| [Development and evidence practice](development-and-evidence.md) | Milestone workflow, CI, smoke worlds and the fifty-year dashboard, results, limitations and how to extend this journal |

## Keeping it current

When implementation materially changes, update the short status in the root [README](../../README.md), the [plan](../../PROJECT_PLAN.md), and the relevant page here. Journal an actual development step after it lands: record its intent, source changes, decision or trade-off, checks run, observed result, and remaining uncertainty. Do not describe a proposed feature as implemented or a passing fixture as proof of real-world fidelity.

The plan remains authoritative for scope and milestone decisions. ADRs remain authoritative for decisions expensive to reverse. The Rust and TypeScript source remains authoritative for runtime behavior.
