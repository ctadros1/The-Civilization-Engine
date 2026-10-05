<p align="center"><img src="assets/previews/civilization-engine-mark.svg" width="88" height="88" alt="The Civilization Engine mark"></p>
<h1 align="center">The Civilization Engine</h1>
<p align="center"><strong>A bottom-up simulation of how people build a civilization.</strong></p>
<p align="center">People make a living, form families, trade, learn and build. Their choices shape the settlements and societies that emerge.</p>

<p align="center">
  <a href="#the-project">Project</a> · <a href="#current-status">Status</a> · <a href="#technical-overview">Technical overview</a> · <a href="#run-it">Run it</a> · <a href="#documentation">Documentation</a>
</p>
<p align="center">
  <img src="https://img.shields.io/badge/milestone-M3b%20complete%20%7C%20M3c%20in%20progress-9a6a36?style=flat-square" alt="M3b complete; M3c in progress">
  <img src="https://img.shields.io/badge/kernel-Rust-315f55?style=flat-square" alt="Simulation kernel: Rust">
  <img src="https://img.shields.io/badge/observer-TypeScript%20%2B%20PixiJS-526c83?style=flat-square" alt="Observer: TypeScript and PixiJS">
</p>

## The project

The Civilization Engine (TCE) is a single-player simulation of societies growing from the everyday decisions of their people. Its long-term vision spans early settlements through modern civilization. The guiding principle is **the engine authors the vocabulary, never the plot**: it provides people, resources, institutions and rules, while the simulation decides what societies become.

The current build simulates an early farming society in a local web observer. Modern industry and cities are a longer-term ambition, not features of the current build or the planned v1 scope. Unreal Engine support is planned; the portable Rust kernel and its C interface are in place, while the Unreal client remains outstanding.

## Current status

| Milestone | Status |
| --- | --- |
| M0–M1: world, people and settlement | Implemented: seeded landscapes, households, farming, buildings, family life, save/load and a web observer |
| M2: Unreal client | In progress: kernel library and C ABI implemented; Unreal integration remains |
| M3a–M3b: economy, knowledge and buildings | Implemented |
| M3c: seasons and time | In progress: accelerated time and the 50-year sanity dashboard are implemented; daily weather and field-water systems are underway |
| M4–M8: law, neighbors and cities | Planned |

The project plan tracks the current slice, detailed implementation boundaries and known limitations. Smoke worlds and the dashboard are engineering checks against selected conditions, not proof that the simulation reproduces history.

## Current build

This is the current web observer: a village with households, buildings, fields and the tools to inspect what people know and how they build.

<p align="center"><img src="assets/m3b/m3b-village.jpg" alt="The Civilization Engine web observer showing an early farming settlement with households and buildings" width="900"></p>

[View the knowledge panel](assets/m3b/m3b-knowledge.jpg) · [Watch the M1 ten-year demo](assets/m1/m1-demo.webm) · [See the M3a economy screenshots](assets/m3a/)

The following generated images illustrate the longer-term scope; they are concept art, not screenshots or implemented features.

<p align="center">
  <img src="assets/previews/civilization-engine-hero.png" alt="Illustrative concept art of a settlement growing into a historical city" width="48%">
  <img src="assets/previews/civilization-engine-modern.png" alt="Illustrative concept art of a modern river metropolis" width="48%">
</p>

## Technical overview

| Area | How it works |
| --- | --- |
| Kernel | Rust workspace with separate crates for world generation, land, people, content, simulation, hosting and the C interface |
| World | Seeded terrain, drainage, lakes and rivers, refined to 8 m simulation cells; generation is reproducible per build and input |
| Simulation | Event-scheduled activities plus calendar cadences; the kernel owns state and decisions, while observers render and forward commands |
| Content | Strict TOML packs compiled and cross-validated before a world starts |
| Observer and boundary | TypeScript, Vite and PixiJS over localhost WebSocket; versioned FlatBuffers messages and a shared frame envelope; C ABI for future Unreal integration |
| Saves | Checksummed, versioned snapshots with explicit migrations; the simulation is not intended to replay deterministically |
| Evidence | Rust and browser checks, selected smoke worlds, and a five-world, fifty-year sanity dashboard |

## Run it

Install the pinned Rust toolchain and Node.js 22, then run from the repository root:

```sh
./tools/run.sh          # macOS and Linux
```

On Windows, run `tools\run.ps1` in PowerShell. The local observer opens at <http://127.0.0.1:7420/>. Development commands and change requirements are in [AGENTS.md](AGENTS.md).

## Documentation

- [Project plan and current milestone status](PROJECT_PLAN.md)
- [Development journal and technical notes](docs/development-journal/README.md) (historical source snapshot; see the project plan for current status)
- [Architecture decision records](decisions/README.md)
- [Research index](research/README.md)
- [Content authoring guide](content/README.md)
- [Web observer guide](web/README.md)

## License

No license has been added. Until one is chosen, the repository does not grant reuse or redistribution rights.
