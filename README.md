<p align="center"><img src="assets/previews/civilization-engine-mark.svg" width="96" height="96" alt="Civilization Engine mark"></p>
<h1 align="center">The Civilization Engine</h1>
<p align="center"><strong>An endless, emergent simulation of human civilization.</strong></p>
<p align="center">People build settlements, cities, institutions, economies, technologies, and ways of life across history, from the earliest communities through modern civilization.</p>

<p align="center">
  <a href="#vision">Vision</a> · <a href="#visual-previews">Visual previews</a> · <a href="#simulation-design">Simulation design</a> · <a href="#project-status">Status</a> · <a href="#explore-the-repository">Explore</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-planning-9a6a36?style=flat-square" alt="Project status: planning">
  <img src="https://img.shields.io/badge/engine-Rust%20%2B%20Unreal%20Engine-315f55?style=flat-square" alt="Planned engine: Rust and Unreal Engine">
  <img src="https://img.shields.io/badge/research-170%20topics-526c83?style=flat-square" alt="170 research topics">
</p>

## Vision

The Civilization Engine is a personal project to build a deep, plausible simulation of people and the societies they create. Individuals make decisions, form households and communities, work, trade, invent, govern, build, migrate, and adapt. Settlements can grow into cities, while institutions and technologies emerge from the actions of their people.

The guiding principle is **the engine authors the vocabulary, never the plot**. The simulation supplies people, needs, resources, institutions, materials, and rules. It does not dictate a storyline or require a society to pass through fixed historical eras. Political and cultural labels are interpretations of what people have built, not switches that set the world into a predetermined state.

## Visual previews

**Illustrative concept art, generated for this README.** These images communicate the intended scope across historical and modern settings. They are not screenshots or evidence of implemented simulation features.

<table>
  <tr>
    <td width="50%"><img src="assets/previews/civilization-engine-hero.png" alt="Illustrative concept of a river settlement growing into a historical city"><br><strong>Settlements and cities across history</strong></td>
    <td width="50%"><img src="assets/previews/civilization-engine-modern.png" alt="Illustrative concept of a contemporary metropolis with transit, bridges, and older districts"><br><strong>Modern civilization and urban systems</strong></td>
  </tr>
</table>

## Simulation design

| System | Direction |
| --- | --- |
| People | Individuals with needs, routines, relationships, memory, and bounded decision-making |
| Settlements | Communities that choose where and how to grow, shaping neighborhoods and cities over time |
| Society | Governance, law, culture, religion, education, social groups, and conflict |
| Economy | Production, labor, property, markets, money, firms, trade, and public services |
| Technology | Knowledge and inventions that spread, change, and sometimes disappear |
| World | Terrain, climate, water, ecology, resources, hazards, and infrastructure |
| Observer | A detailed view into people, places, institutions, decisions, and the history they produce |

### Architectural direction

The plan separates the simulation kernel from its presentation. A data-oriented Rust kernel is intended to own world state and simulation logic; an Unreal Engine client will render the world and provide the observer experience. A headless host and browser-based inspection tools are also part of the planned architecture. These are design decisions in the project plan, not code that exists in this repository yet.

## Project status

**Planning.** No simulation or application has been implemented yet. The repository currently contains the project plan and a growing body of research intended to inform future development. Read the plan for the full scope, architecture, milestones, risks, and decision log.

## Explore the repository

| Start here | For |
| --- | --- |
| [Project plan](PROJECT_PLAN.md) | Vision, scope, architecture, milestones, risks, and decisions |
| [Research index](research/README.md) | Research topics, domains, and milestone priorities |
| [`research/`](research/) | Reports and prompts across 16 simulation domains |

The research covers simulation engineering, reference games, world generation, people, demography, culture, technology, economics, government, cities, architecture, infrastructure, diplomacy, rendering, observer experience, and validation.

## Project boundaries

This is a personal simulation project rather than an academic research instrument. Realism means plausible behavior checked against selected historical patterns, not proof that the simulation reproduces history. The plan records open questions and decisions as development progresses.

## License

No license has been added yet. Until a license is chosen, reuse and redistribution are not granted by this repository.
