# Development Journal and Technical Guide

This folder explains how TCE has been built, how its current simulation works, and where the main code boundaries are. It complements the milestone plan, architecture decision records (ADRs), code comments, and research library; it does not replace them.

## Source baseline

These pages describe the public `main` tree at commit [`fad23fa`](https://github.com/ctadros1/The-Civilization-Engine/commit/fad23fa), merged on 2026-10-04. Its current development point is M3b, with the first deposit-placement primitive started in slice Q. Treat claims about implementation as belonging to that source baseline. Later commits may change the code or milestone status; check the current source and plan before relying on a detail.

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
