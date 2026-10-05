<p align="center"><img src="assets/previews/civilization-engine-mark.svg" width="88" height="88" alt="The Civilization Engine mark"></p>
<h1 align="center">The Civilization Engine</h1>
<p align="center"><strong>A bottom-up simulation of how people build a civilization.</strong></p>
<p align="center">Individuals make a living, form families, trade, learn, build and adapt. Their choices can grow settlements into cities and shape the societies around them.</p>

<p align="center">
  <a href="#the-project">Project</a> · <a href="#current-build">Current build</a> · <a href="#technical-overview">Technical overview</a> · <a href="#run-it">Run it</a> · <a href="#documentation">Documentation</a>
</p>
<p align="center">
  <img src="https://img.shields.io/badge/current-M3b%20(Q%20underway)-9a6a36?style=flat-square" alt="Current milestone: M3b, slice Q underway">
  <img src="https://img.shields.io/badge/kernel-Rust-315f55?style=flat-square" alt="Simulation kernel: Rust">
  <img src="https://img.shields.io/badge/observer-TypeScript%20%2B%20PixiJS-526c83?style=flat-square" alt="Observer: TypeScript and PixiJS">
</p>

## The project

The Civilization Engine (TCE) is a single-player simulation project about societies that grow from the everyday decisions of their people. Its long-term scope runs from early settlements to modern civilization. The guiding rule is **the engine authors the vocabulary, never the plot**: people act within a world of authored rules and possibilities; the simulation does not force a historical storyline or predetermined outcomes.

The project is actively developed. The working build is a Rust simulation viewed through a local web observer. It currently models an early farming village; modern civilization is a long-term goal, not an implemented feature. Unreal Engine support is planned, with the kernel's C interface in place and the Unreal client still to be built.

| Milestone | Status |
| --- | --- |
| M0 foundations and web observer | Implemented |
| M1 people, households, farming and huts | Implemented |
| M2 Unreal client | In progress; kernel interface implemented |
| M3a village economy | Implemented |
| M3b knowledge and buildings | In progress; slices M–P implemented, deposit placement begun in Q |
| M3c seasons and accelerated time; M4–M8 | Planned |

## Current build

This is a screenshot from the running web observer: an early farming village with fields, people, trails and market data.

<p align="center"><img src="assets/m3a/m3a-village.jpg" alt="Current TCE web observer showing a farming village by a river, with fields, households and market data" width="900"></p>

[Watch the M1 ten-year simulation demo](assets/m1/m1-demo.webm) · [See the M3a village and economy screenshots](assets/m3a/)

The following modern-city image is generated concept art for the long-term scope. It is not a screenshot or a claim about the current build.

<p align="center"><img src="assets/previews/civilization-engine-modern.png" alt="Illustrative concept art of a modern river metropolis" width="760"></p>

## Technical overview

| Area | Implementation |
| --- | --- |
| Simulation | Rust workspace; event-scheduled people plus minute-to-year system cadences |
| World | Seeded terrain, drainage, lakes and rivers; 8 m simulation cells |
| Content | Strict TOML packs compiled and cross-validated before a world starts |
| Observer | TypeScript, Vite and PixiJS; map and panels read kernel state and submit commands |
| Boundaries | Versioned FlatBuffers payloads in a shared frame envelope; localhost WebSocket today, C ABI for future Unreal integration |
| Saves | Checksummed, versioned snapshot generations with explicit schema migrations |

The kernel owns simulation state. The observer renders it and forwards user commands. See the [technical docs](docs/development-journal/) for the data flow, simulation model, save format, interfaces, and design history.

## Run it

Install the pinned Rust toolchain and Node.js 22, then run from the repository root:

```sh
./tools/run.sh          # macOS and Linux
```

On Windows, run `tools\run.ps1` in PowerShell. The host opens the local observer at <http://127.0.0.1:7420/>. For development commands, tests and troubleshooting, see [AGENTS.md](AGENTS.md).

## Documentation

- [Development journal and technical guide](docs/development-journal/README.md)
- [Project plan and milestone decisions](PROJECT_PLAN.md)
- [Architecture decision records](decisions/README.md)
- [Research index: 170 topics across 16 domains](research/README.md)
- [Content authoring guide](content/README.md)
- [Web observer guide](web/README.md)

## License

No license has been added. Until one is chosen, the repository does not grant reuse or redistribution rights.
