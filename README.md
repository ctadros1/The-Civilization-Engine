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
  <a href="#project-status"><img src="https://img.shields.io/badge/M10d-money%20systems%20underway-b57932?style=flat-square" alt="Milestone M10d money systems underway"></a>
  <a href="#technical-highlights"><img src="https://img.shields.io/badge/kernel-Rust-315f55?style=flat-square" alt="Simulation kernel: Rust"></a>
  <a href="#technical-highlights"><img src="https://img.shields.io/badge/observer-TypeScript%20%2B%20PixiJS-526c83?style=flat-square" alt="Observer: TypeScript and PixiJS"></a>
</p>

## The project

The Civilization Engine (TCE) is a long-running simulation of people, households and settlements. It provides resources, skills, institutions and physical systems; people make choices within them, so the engine does not prescribe a civilization's story or destination.

The current observer presents an early farming world. M10 adds industrial systems and services; its money systems are underway. Modern civilization and the Unreal client remain future work.

## Project status

| Status | Milestone | Overview |
| --- | --- | --- |
| <img src="assets/icons/status-complete.svg" width="18" alt=""> **Complete** | M0–M9 | Settlement life, institutions, cities and many-settlement simulation. |
| <img src="assets/icons/status-complete.svg" width="18" alt=""> **Complete** | M10a–M10c | Industry, transport, services and public health. |
| <img src="assets/icons/status-in-progress.svg" width="18" alt=""> **In progress** | M10d | Household money and redeemable notes. |
| <img src="assets/icons/status-planned.svg" width="18" alt=""> **Planned** | M10d | Polity-issued notes, counterfeiting and the complete M10 demo. |
| <img src="assets/icons/status-planned.svg" width="18" alt=""> **Planned** | Future scope | Modern-era systems and the Unreal client. |

See the [project plan](PROJECT_PLAN.md) and [development journal](docs/development-journal/README.md) for milestone details and implementation evidence.

## Latest live demo: M9, many settlements

The newest committed observer video is M9. It shows one settlement in detail while households resolve daily work across other settlements, with a followed resident and a climate change. The M10d code has advanced, but no M10 observer video has been captured; the complete demo remains planned. The gallery distinguishes recorded captures from run reports and planned demos.

<p align="center"><a href="demos/m9/m9-overview.mp4"><img src="demos/m9/m9-overview-poster.png" alt="M9 live demo: many settlements at household resolution" width="900"></a></p>

[Watch the titled M9 demo](demos/m9/m9-overview.mp4) · [Ten-year M9 run report](demos/m9/m9-demo.txt) · [Video-capture run report](demos/m9/m9-capture-run.txt) · [All demo versions](demos/README.md)

These generated previews show the project's longer-term historical and modern scope. They are concept art, not screenshots of implemented features.

<p align="center">
  <img src="assets/previews/civilization-engine-hero.png" alt="Concept art of a historical settlement growing into a city" width="48%">
  <img src="assets/previews/civilization-engine-modern.png" alt="Concept art of a modern river metropolis" width="48%">
</p>

## Technical highlights

- **Kernel and observer:** Rust simulation workspace; TypeScript and PixiJS web observer; versioned FlatBuffers boundary and portable C interface.
- **Emergent simulation:** event-scheduled people and household economies, authored building grammars, institutions, weather, water, disease, conflict and transport.
- **Many-settlement scale:** each person remains individually represented; settlements away from the observer's focus can resolve household work once per day.
- **Industrial systems:** river-powered installations drive recipes; households can build mills, mines and works, lay rail, operate trains, and produce and distribute gas light.
- **Persistence:** checksummed versioned snapshots with explicit migrations. Current boundaries: wire 1.106, save schema 214 and content API 195.

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

Dedicated Linux VMs on servernode1 and servernode3, with optional capacity on servernode4, provide the [GitHub Actions runner pool](docs/operations/github-runners.md). Automatic routing prefers idle node4, then node3 and node1; jobs budgeted above six hours exclude node4, and timing benchmarks stay on node3. Windows and pull-request checks retain GitHub-hosted runners. The runner guide covers candidate branches, routing, isolation, and safely stopping the optional runner before switching servernode4 to Windows.
