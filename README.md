<p align="center"><img src="assets/previews/civilization-engine-icon.png" width="88" height="88" alt="The Civilization Engine icon"></p>
<h1 align="center">The Civilization Engine</h1>
<p align="center"><strong>A bottom-up simulation of how people build a civilization.</strong></p>
<p align="center">People make a living, raise families, trade, learn, build and govern. Their choices shape the settlements and institutions that emerge.</p>

<p align="center">
  <a href="#the-project">Project</a> · <a href="#current-status">Status</a> · <a href="demos/README.md">Demo gallery</a> · <a href="#technical-highlights">Technical highlights</a> · <a href="#run-it">Run it</a> · <a href="#documentation">Documentation</a>
</p>
<p align="center">
  <img src="https://img.shields.io/badge/milestone-M6a%20complete%20%C2%B7%20M6b%20in%20progress-9a6a36?style=flat-square" alt="M6a complete; M6b in progress">
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
| M5c · Inter-polity relations and public works | Implemented: claims, agreements ratified by each settlement's own law, carried payments, and household and village log crossings. A failed ratification appears in a test world, not a lived demo. |
| M6a · Water and sickness | Implemented: flowing water, aquifers, springs and household wells; waterborne contamination and disease; illness, care, suspicion, source avoidance and collective responses. The cholera demo traces an outbreak that people do not connect to the wells that carried it. |
| M6b · Fire, masonry and floods | Done (its open items close this entry): buildings burn and spread fire; people notice, raise alarms, carry water, strip threatened roofs and salvage goods; those latter choices remain available without a known water route. The dashboard has a Fires row for household escapes and a preliminary ungraded Floods and storms row; its exposure accounting remains partial. Masonry, brick and tile, flood depths and wet-floor damage, evacuation, storm uplift and flood/storm kernel commands are implemented. River overrides hold their requested days and route added flow downstream; forced storms change today's wind. Hazard origins and one uplift evaluation per covering per storm are saved. Flood exposure and floor loss advance daily; household supplies return to the chosen flood refuge. The chronicle sums household fire escapes monthly. An equipped crew can pull down an empty threatened building with ownership or leave on site; partial work saves. People tell dated accounts of buildings found burning through ordinary contact, without asserting a cause. Fire pages add dated notices and response work with physical reasons. Task awareness follows personal records and connected physical fire scenes. The Help at a fire lens separates one current availability probe from thirty days of recorded work. The 1–100/K household-year ignition sweep is measured and published; the Fires row separates observer-caused losses. Crew coordination and response calibration remain partial. BG begins with saved sub-grid bank crests, explicit borrow-earth accounting and conservative river-flood barriers, shown as lines in the observer. Ground beneath banks and borrow pits enters flood storage, including after a breach. Actual overtopping lowers bank crests and retains washed earth as sediment stock. Households remember what floods cost them; a gathering can pass a flood bank where it would have kept that water out, and people choose to dig, carry and place its earth in equal shares (one-segment banks on free ground only). Banks settle; water held against one side seeps through and can break them, and a broken bank is told in the chronicle. A gathering can pass a law to keep a bank it built, which asks the households to mend it after floods and each year; nobody mends a bank no law keeps. A household whose crop went short of water can dig an irrigation ditch to its field from the river and open it in dry spells; its water is debited from the river below and every loss is accounted for, and the map draws it. Households remember fires they lost a building to or saw; once someone knows how to fire tile, a gathering can pass a rule that new roofs near other buildings must not burn, which builders keep with tile or a plot apart, or break. Households also hear of buildings that give way; a gathering can ask that new walls be of stone or brick, or that new buildings stand apart. The M6b demo sets a fire in a village that had none of its own: it takes three huts, and the gathering passes walls of stone and a gap between buildings within two weeks, which bind nothing in the year after because the huts are mended rather than rebuilt. Left open: trapped rain, shared ditches, multi-segment banks, crossing scour and flood gates. |
| M6c · War and conquest | Implemented: households take from homes of other villages they have seen; bands are called, joined, mustered and walked to a household, where a two-sided encounter by the minute decides who gives way and what is carried off; a gathering may send a band by law, raise a palisade with gates kept shut, hear raids its own people made, seek terms with a word not to raid, take a village its band holds into one polity, and lose it again to a revolt there. The Government panel shows each polity's settlements and a siege page with each side's food at current use. In the lived demo two villages of 100 raided each other for a year and a half; Heronfield raised a palisade, made a law against raiding, took its custom to its elders and sent bands by law until one held Willowbank (two priors nudged, logged). Neither gathering took the held village in. |
| M7 · Politics deepens | In progress (M7a, councils and elections, and M7b, courts and precedent, implemented): offices may be held for a term and filled by lot from those who stood for them; a village may amend its custom to a council of 5 or 9 seats filled by election or by lot, whose election is called, told, stood for, polled at the hearth and counted by the block vote, every failure recorded; the council alone is the body then, and the label reads it as a republic. Those who stand say where they stand and campaign at the hearth; voters weigh distance, record and their faction's slate; a polity of several settlements may share its seats among them; factions that would lose apart may agree one slate. The M7a demo's village nudged one prior and took a council of 5 by 65 to 0, eight standing for its seats ([ADR-0024](decisions/0024-seats-offices-and-elections.md)). A law may name a forum to hear cases (one hearer, a panel by lot or the elders), whose findings are remembered and told and lean later sittings as its authority says; a limit adopted through the custom's amendment route forbids laws that keep people from daylight work or levy too much, and a forum given review may strike a breaching law down when someone it burdens challenges it; a ruling is known person by person, and what a forum finds is held against the forum ([ADR-0025](decisions/0025-forums-precedent-and-review.md)). M7c under way ([ADR-0026](decisions/0026-organizations-privileges-and-trade-policy.md)): a polity may lay a duty on tools its people buy at another settlement's door, paid to its store or kept back, or an embargo on a settlement's doors, kept or broken person by person. Those who sell the goods a duty would serve may organize, by rules of who is admitted and who decides; those turned away hold it against the organization, and a polity may recognize one, sparing its members its duties on those goods. A recognized organization may be granted a licence or a staple: others keep it or break it at their doors, its members bring breaches as cases, and a workshop that keeps it and sells nothing is given up as barred. A store's keeper may draw from it for their own household and, when it runs low, answer those they favour; a duty's collector may ask a buyer for a payment to assess nothing; a sponsor whose law would fail may give food to households it would bring round, who owe them for it at the gathering. A custom may be amended to close the gathering to members of recognized or privileged organizations, and the government panel says in sentences what an organization has come to hold. The M7c demo's villages passed an embargo and a duty on tools once a neighbour's door was used, but nobody organized: at this scale trade is too small to fight over. M7d under way ([ADR-0027](decisions/0027-claims-and-coin.md)): a household short of food before its harvest may borrow grain from a neighbour with food to spare, who weighs the loan by what it believes of the borrower and asks an increment; the loan is repaid after the harvest or, unpaid, defaulted and remembered. A lender that would refuse may lend against the use of a field or with a guarantor; at the due day a debtor pays, asks for time or refuses, and a failed loan's pledge is taken. A workshop short of a wage owes it and fails only if it is still unpaid when due. Next, *debts heard*. |

The web observer currently presents an early farming world. The project's longer vision reaches modern civilization, but modern industry, cities and the Unreal client are not implemented. Smoke runs and dashboards check selected behaviors and engineering invariants; they do not prove historical realism.

## Latest live demo: M6a water and sickness

The newest completed milestone demo follows a cholera outbreak through village wells and shows what residents and their polity believed about its source. These are captures from the live observer; the [demo gallery](demos/README.md) collects this run and the earlier milestone demos.

<p align="center"><a href="demos/m6a/m6a-overview.mp4"><img src="demos/m6a/m6a-overview-poster.png" alt="M6a live demo: a village by the water, with the first chapter title" width="900"></a></p>

[Watch the 1080p M6a demo](demos/m6a/m6a-overview.mp4) · [Incidents panel capture](demos/m6a/m6a-incidents.png) · [All demo versions](demos/README.md)

These generated previews illustrate the project's longer-term historical and modern scope. They are concept art, not screenshots of implemented features.

<p align="center">
  <img src="assets/previews/civilization-engine-hero.png" alt="Concept art of a historical settlement growing into a city" width="48%">
  <img src="assets/previews/civilization-engine-modern.png" alt="Concept art of a modern river metropolis" width="48%">
</p>

## Technical highlights

- **Simulation kernel:** Rust workspace organized around world generation, changing land, people, authored content, simulation composition, hosting and a C interface.
- **Emergent systems:** event-scheduled daily lives and calendar cadences; goods move through an accounting ledger; building grammars expand saved designs; institutions emerge through proposals, deliberation, law and collective action. Waterborne disease and fire spread through household choices, weather, buildings and local response.
- **Multiple settlements:** settlement identity and residence histories are explicit. Households learn places by contact and choose whether to visit, move or found elsewhere.
- **Content and boundaries:** strict TOML packs (content API 117) feed the kernel. The web observer uses a localhost WebSocket and versioned FlatBuffers messages (TCE wire 1.82); it displays kernel state and forwards commands.
- **Persistence:** checksummed, versioned snapshots (schema 128) with explicit migrations. World generation is reproducible for a seed on one build; simulation replay is not a goal.

## Run it

Install the pinned Rust toolchain and Node.js 22, then launch from the repository root:

```sh
./tools/run.sh          # macOS and Linux
```

On Windows, run `tools\run.ps1` in PowerShell. The observer opens at <http://127.0.0.1:7420/>. See [AGENTS.md](AGENTS.md) for development commands and change requirements.

## Documentation

- [Project plan and milestone status](PROJECT_PLAN.md)
- [Development journal and technical guide](docs/development-journal/README.md)
- [Demo gallery across milestones](demos/README.md)
- [Institutions, law and social change](docs/development-journal/institutions-and-order.md)
- [Settlements, movement and trade](docs/development-journal/settlements-and-exchange.md)
- [Inter-polity relations and public works](docs/development-journal/relations-and-public-works.md)
- [Water, sickness and fire systems](docs/development-journal/hazards-and-health.md)
- [Architecture decision records](decisions/README.md)
- [Research index](research/README.md)
- [Content authoring guide](content/README.md)
- [Web observer guide](web/README.md)

## License

No license has been added. Until one is chosen, the repository does not grant reuse or redistribution rights.
