<p align="center"><img src="assets/previews/civilization-engine-mark.svg" width="88" height="88" alt="The Civilization Engine mark"></p>
<h1 align="center">The Civilization Engine</h1>
<p align="center"><strong>A bottom-up simulation of how people build a civilization.</strong></p>
<p align="center">People make a living, raise families, trade, learn, build and govern. Their choices shape the settlements and institutions that emerge.</p>

<p align="center">
  <a href="#the-project">Project</a> · <a href="#current-status">Status</a> · <a href="#technical-highlights">Technical highlights</a> · <a href="#run-it">Run it</a> · <a href="#documentation">Documentation</a>
</p>
<p align="center">
  <img src="https://img.shields.io/badge/milestone-M5%20complete%20%C2%B7%20M6%20planned-9a6a36?style=flat-square" alt="M5 complete; M6 planned">
  <img src="https://img.shields.io/badge/kernel-Rust-315f55?style=flat-square" alt="Simulation kernel: Rust">
  <img src="https://img.shields.io/badge/observer-TypeScript%20%2B%20PixiJS-526c83?style=flat-square" alt="Observer: TypeScript and PixiJS">
</p>

## The project

The Civilization Engine (TCE) is a single-player simulation about societies growing from ordinary lives. People respond to food, work, family, resources, neighbors and institutions; the simulation lets settlements and social systems take shape from those choices.

Its guiding rule is **the engine authors the vocabulary, never the plot**. Content supplies people, goods, techniques, offices and policies. It does not prescribe eras, storylines or outcomes. The long-term vision spans early communities through modern civilization. The current simulation is focused on early farming societies; modern industry and the Unreal client are future work.

## Current status

| Area | Status |
| --- | --- |
| M0–M1 · World and village life | Implemented |
| M2 · Unreal client | The portable Rust kernel and C interface are available; Unreal integration remains future work. |
| M3a–M3c · Economy, knowledge, buildings, weather and soils | Implemented |
| M4 · Councils, law, crime and political change | Implemented. The latest fifty-year dashboard still has a failed food-price row in one world; see the project plan for the result and limits. |
| M5a · Neighboring settlements | Implemented: contact, visits, marriage, migration and coalition founding. The 3,000-person, three-settlement run takes about 19.6 minutes at Max; the ten-minute design budget is not met. |
| M5b · Trade and diffusion | Implemented: households trade using dated reports, fetch goods to resell, and learn techniques and building styles through contact. The earlier roof-style demo no longer qualifies after corrected river routing separated its villages. |
| M5c · Inter-polity relations and public works | Implemented: claims, agreements ratified by each settlement's own law, carried payments, and household and village log crossings. A failed ratification appears in a test world, not a lived demo; enclosures moved to M6, where force gives them a purpose. |

The web observer currently presents an early farming world. Cities and modern civilization are long-term scope, not features of the current build or planned v1. Smoke runs and dashboards check selected behaviors and engineering invariants; they do not prove historical realism.

## What the build looks like

This village screenshot predates the newer government and multi-settlement systems. It illustrates the simulation's current visual foundation; the journal and plan describe the broader integrated feature set.

<p align="center"><img src="assets/m3b/m3b-village.jpg" alt="The web observer showing an early farming village with homes and fields" width="900"></p>

[Knowledge panel](assets/m3b/m3b-knowledge.jpg) · [M1 village demo](assets/m1/m1-demo.webm) · [M3a economy screenshots](assets/m3a/)

These generated previews illustrate the project's longer-term historical and modern scope. They are concept art, not screenshots of implemented features.

<p align="center">
  <img src="assets/previews/civilization-engine-hero.png" alt="Concept art of a historical settlement growing into a city" width="48%">
  <img src="assets/previews/civilization-engine-modern.png" alt="Concept art of a modern river metropolis" width="48%">
</p>

## Technical highlights

- **Simulation kernel:** Rust workspace organized around world generation, changing land, people, authored content, simulation composition, hosting and a C interface.
- **Emergent systems:** event-scheduled daily lives and calendar cadences; goods move through an accounting ledger; building grammars expand saved designs; institutions emerge through proposals, deliberation, law and collective action.
- **Multiple settlements:** settlement identity and residence histories are explicit. Households learn places by contact and choose whether to visit, move or found elsewhere.
- **Content and boundaries:** strict TOML packs (content API 68) feed the kernel. The web observer uses a localhost WebSocket and versioned FlatBuffers messages (TCE wire 1.60); it displays kernel state and forwards commands.
- **Persistence:** checksummed, versioned snapshots (schema 71) with explicit migrations. World generation is reproducible for a seed on one build; simulation replay is not a goal.

## Run it

Install the pinned Rust toolchain and Node.js 22, then launch from the repository root:

```sh
./tools/run.sh          # macOS and Linux
```

On Windows, run `tools\run.ps1` in PowerShell. The observer opens at <http://127.0.0.1:7420/>. See [AGENTS.md](AGENTS.md) for development commands and change requirements.

## Documentation

- [Project plan and milestone status](PROJECT_PLAN.md)
- [Development journal and technical guide](docs/development-journal/README.md)
- [Institutions, law and social change](docs/development-journal/institutions-and-order.md)
- [Settlements, movement and trade](docs/development-journal/settlements-and-exchange.md)
- [Inter-polity relations and public works](docs/development-journal/relations-and-public-works.md)
- [Architecture decision records](decisions/README.md)
- [Research index](research/README.md)
- [Content authoring guide](content/README.md)
- [Web observer guide](web/README.md)

## License

No license has been added. Until one is chosen, the repository does not grant reuse or redistribution rights.
