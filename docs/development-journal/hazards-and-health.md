# Water, Sickness and Fire

This guide describes the integrated `main` implementation as of 2026-10-22. The project plan
records milestone status and run evidence; the architecture decisions define durable boundaries.
The detailed slice-by-slice record is in the [development history](development-history.md).

## Ownership and boundaries

The kernel owns the state and causal rules. `civ-land` owns hydrology, mutable water features and
wells; `civ-agents` owns disease, contamination, household knowledge, suspicion and fire behavior.
`civ-sim` composes those domains, saves authoritative state and builds observer frames. `civ-host`
validates commands and queries. The web observer renders the returned state and forwards explicit
interventions. It does not simulate exposure, choose sources or determine outcomes.

Authored disease and fire parameters live in content profiles. The observer can introduce a disease,
hold a drought or set a building alight through named commands. Those interventions are recorded;
ordinary illness and fire behavior still follow the simulation's own rules. The M6 boundary at this
tree is TCE wire 1.68, save schema 92 and content API 85.

## Water and wells

The daily water balance connects weather and soils to the river network. Water beyond the reference
soil's capacity recharges a shallow aquifer or becomes runoff. Aquifer heads exchange with
neighboring patches and river reaches; seepage can expose a spring. A runoff store and river
recession scale each reach's daily flow against its mean. The aquifer is saved; springs are derived
from its head and the land surface.

Households choose usable sources by walking distance and availability. A source may be a river or
lake edge, a flowing spring, or a well the household may draw from. Every use records the source,
household, amount and day. A household can assess a well's expected walking and lifting savings,
then dig and line it in stages. Wells meet the water table, can be deepened, refill from the aquifer,
and lose water to draws. Their linings decay and can be repaired; a failed lining collapses the well.
Households can add a lid and curb to a suspected well, which reduces surface runoff entering its
mouth while leaving subsurface seepage intact.

These are coupled but distinct records: aquifer water, well columns and household draws have
explicit accounting. Water at a source is finite for that day. The observer's Water lens shows
either which sources could serve a home or where its residents actually drew during the recent
period; those views use different denominators and time windows.

## Contamination, illness and knowledge

Cholera and bacillary dysentery are authored disease definitions. Each infection is an episode with
its own course, acquisition source, symptoms, severity, shedding, protection and outcome. Exposure
from household contact and drinking water is combined for a person-day and resolved once. Illness
keeps a person abed; household members may choose to tend them. Care and any known treatment are
recorded on the infection, and deaths are identified as disease deaths. Fluid replacement is a
learned technique that reduces cholera's daily fatal risk.

For each disease, the simulation tracks loads in household waste, water bodies, wells and household
water stores. Rain washes part of a household's load toward nearby water. Ground seepage has a
travel time; river loads flow downstream; draws carry a share of source load to the household.
Loads decay, and each day's movement is recorded so an outbreak page can explain modeled sources.
An outbreak groups a disease's cases in one settlement. It opens with the first case while no
outbreak is active and closes after its settlement's active cases and residual household loads end
under the implemented rule.

Truth and knowledge are deliberately separate:

- **Incident facts:** cases, sources, contamination movements, deaths and episode relationships.
- **People's knowledge:** who saw sickness, which source they saw households draw from, and each
  person's suspicion tally from the sources and cases they know.
- **Institutional response:** the issue raised, the law proposed or passed, and whether households
  close a source or undertake a shared well.

Households may pass over a suspected source when they know another, cover their own well, or weigh
moving after illness or suspicion. A polity may close a named source or authorize a well as public
work. Each step uses the existing choice, deliberation and labor systems. It does not automatically
identify the true source. The M6a demo makes this distinction concrete: the model traces illness
through seven wells, but people do not connect those wells to the outbreak; they suspect and close a
spring with no cases.

The shared Incidents panel presents events, what people know and what a polity has heard or done as
separate layers. Incident steps have typed links to causes and episodes. Chronicle entries can open
the incident or its map location. The Water lens exposes access and observed draws; it does not
replace the incident's source accounting.

## Fire and response

Wind and exposed-fuel dryness are daily weather facts drawn on independent random keys so adding
them does not perturb unrelated weather draws. Fire advances in one-minute events only while
something is burning. A fire progresses from a covering catching, through full involvement, to
burnt out. Heat reaches nearby roofs according to roof-edge distance and wind. Each roof accumulates
and sheds exposure; its covering catches according to its kind, condition and dryness. Coverings
burn away, timber members lose section through charring, part of the infill is lost, and goods under
the roof are removed through a dedicated ledger channel. Sleeping people still inside can die when
the building is fully involved. A fire that catches from another records that parent; connected fires
form a conflagration episode.

Ordinary ignitions come from hearths and work that uses fire. A household's hearth is banked when no
one is awake at home; fires used for baking or firing pottery are represented separately. Escapes
are keyed by household building and hour, depend on dryness, and can be put out by someone awake
nearby. An unattended escape has time to grow before it reaches the roof. People can see a burning
building, shout to nearby people, and hear alarms; those records form the fire's knowledge layer.

People who know of a fire weigh choices against their other needs and work. Implemented responses
include carrying a water load from household stores or a source to threatened roofs, stripping a
threatened covering, and carrying household goods out. A fire page records what each response did.
The response set remains partial. M6b also adds masonry, tile, flood and storm damage, banks,
ditches, building rules and a great-fire demo. An equipped crew can pull down an empty threatened
building; work can be partial and saved. Household awareness and response calibration remain open.

The dashboard grades fires that escaped from fires in use against the research reference range of
1–100 per 1,000 household-years. It reports how many were put out, burned a building, spread from a
neighbor or killed, while observer-set fires are counted separately. A green interval means that
one aggregate rate fell within that chosen range; it is not a validation of fire realism. Current
probes show water can reduce losses in a tightly spaced block. Default runs rarely choose to strip a
roof, so calibration and broader behavioral evidence remain open.

## Durable decisions and code map

- [ADR-0021](../../decisions/0021-water-and-sickness.md) defines the water, disease, knowledge and
  response boundaries.
- [ADR-0022](../../decisions/0022-fire-floods-and-storms.md) defines fire, weather and planned
  hazard extensions.
- `kernel/crates/civ-land/src/water.rs` and `wells.rs` own water movement and well state.
- `kernel/crates/civ-agents/src/contagion.rs`, `sickness.rs`, `suspicion.rs` and their
  `population/` modules own exposure, infection, knowledge and household response.
- `kernel/crates/civ-agents/src/fire.rs` and `population/fire.rs` own fire physics, ignitions and
  people's actions.
- `kernel/crates/civ-sim/src/frames/` maps domain facts into water, sickness, coverage, incident and
  fire views; `persist/agents.rs` saves the authoritative records.
- `web/src/water.ts`, `coverage.ts`, `incidents.ts` and the observer panels display those frames.

For exact run commands, measurements, fixes and open questions, see the [M6 entries in the project
plan](../../PROJECT_PLAN.md#9-decisions-log) and the [development history](development-history.md).
