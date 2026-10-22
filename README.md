<p align="center">
  <img src="assets/previews/civilization-engine-icon.png" alt="The Civilization Engine icon" width="128">
</p>

<h1 align="center">The Civilization Engine</h1>

<p align="center"><strong>A living-world simulator where people build their own communities, economies and institutions.</strong></p>

<p align="center">People make a living, raise families, trade, learn, build and govern. Their choices shape the societies that emerge.</p>

<p align="center">
  <a href="#the-project">Project</a> · <a href="#project-status">Status</a> · <a href="demos/README.md">Demo gallery</a> · <a href="#technical-highlights">Technical highlights</a> · <a href="#run-it">Run it</a> · <a href="#documentation">Documentation</a>
</p>
<p align="center">
  <a href="#project-status"><img src="https://img.shields.io/badge/M0%E2%80%93M9-complete-39785b?style=flat-square" alt="Milestones M0 through M9 complete"></a>
  <a href="#project-status"><img src="https://img.shields.io/badge/M10a-in%20progress-b57932?style=flat-square" alt="Milestone M10a in progress"></a>
  <a href="#technical-highlights"><img src="https://img.shields.io/badge/kernel-Rust-315f55?style=flat-square" alt="Simulation kernel: Rust"></a>
  <a href="#technical-highlights"><img src="https://img.shields.io/badge/observer-TypeScript%20%2B%20PixiJS-526c83?style=flat-square" alt="Observer: TypeScript and PixiJS"></a>
</p>

The Civilization Engine (TCE) is a long-running simulation project. It models people, households and settlements from the ground up: how they find food, learn skills, build, trade, make rules and respond to events. The engine supplies the materials and social mechanisms; it does not script a civilization's story or destination.

The current playable world is an early farming society. The project roadmap reaches industrial and modern civilization, but those systems are still being built. The Unreal client is also future work.

## Project status

| Area | Status |
| --- | --- |
| M0–M8 · World, society and growing cities | Complete |
| M9 · Many settlements and household resolution | Complete; 100,000-person scale target not met (see [measured results](PROJECT_PLAN.md#9-decisions-log)) |
| M10a · Power and industry | In progress: water-powered mills and household-built water wheels are implemented; fuel, iron and other works continue |
| Unreal client and modern civilization | Future work |

## Latest live demo: M9, many settlements

The current M9 capture shows the observer following a person in one detailed settlement while the other settlements are lived by household. It also shows the world map, settlement-resolution labels and the recorded climate change. The [demo gallery](demos/README.md) separates the short video capture from the full ten-year M9 run report.

<p align="center"><a href="demos/m9/m9-overview.mp4"><img src="demos/m9/m9-overview-poster.png" alt="M9 live demo: many settlements at household resolution" width="900"></a></p>

[Watch the titled M9 demo](demos/m9/m9-overview.mp4) · [Ten-year M9 run report](demos/m9/m9-demo.txt) · [Video-capture run report](demos/m9/m9-capture-run.txt) · [All demo versions](demos/README.md)

These generated previews show the project's longer-term historical and modern scope. They are concept art, not screenshots of implemented features.

<p align="center">
  <img src="assets/previews/civilization-engine-hero.png" alt="Concept art of a historical settlement growing into a city" width="48%">
  <img src="assets/previews/civilization-engine-modern.png" alt="Concept art of a modern river metropolis" width="48%">
</p>

## Technical highlights

- **Simulation kernel:** Rust workspace with separate crates for world generation, land, people, content, simulation, hosting and a versioned C interface.
- **Emergent systems:** event-scheduled lives and household economies; authored building grammars; institutions formed through proposals and collective decisions; weather, water, illness, fire, conflict and transport.
- **Many-settlement scale:** each person remains individually represented. Settlements away from the observer's focus can resolve household work once per day, while the selected settlement continues in detail. Time skips use the same household machinery.
- **Industrial transition underway:** water-powered work is represented by saved installations whose daily output follows river discharge; powered recipes and water mills can be built when households expect them to repay their cost.
- **Persistence and interfaces:** checksummed, versioned snapshots; generated FlatBuffers messages; a localhost WebSocket observer that displays kernel state and forwards commands. Current boundary versions: wire 1.96, save schema 174 and content API 160.

## Run it

Install the pinned Rust toolchain and Node.js 22, then launch from the repository root:

```sh
./tools/run.sh          # macOS and Linux
```

On Windows, run `tools\run.ps1` in PowerShell. The observer opens at <http://127.0.0.1:7420/>. See [AGENTS.md](AGENTS.md) for development commands and change requirements.

## Documentation

- [Project plan and milestone status](PROJECT_PLAN.md)
- [Development journal and technical guides](docs/development-journal/README.md)
- [Demo gallery across milestones](demos/README.md)
- [Architecture decision records](decisions/README.md)
- [Research index](research/README.md)
- [Content authoring guide](content/README.md)
- [Web observer guide](web/README.md)

## License

No license has been added. Until one is chosen, the repository does not grant reuse or redistribution rights.

## CI runner pool

Dedicated Linux VMs on servernode1 and servernode3, with optional capacity on servernode4, provide the [GitHub Actions runner pool](docs/operations/github-runners.md). Trusted push checks and scheduled long runs can use these machines; Windows and pull-request checks retain GitHub-hosted runners. The runner guide covers candidate branches, routing, isolation, and safely stopping the optional runner before switching servernode4 to Windows.
