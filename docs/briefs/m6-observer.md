# M6 design brief: incidents, coverage and the observer

**Scope.** Plan §7's M6 ("Towns & their troubles") lists "incident panels, coverage maps", "weather (visuals and sim)" and "god tools: fire, plague, flood, drought, storm". It is to prove "the incident-plus-aggregate service UX", and its demo is "a cholera outbreak traced to a well; a great fire leads to a proposed masonry code; a siege ends in annexation". Plan §5.4 describes the view: incidents are "sim entities with a lifecycle", and "the dashboards (response time distribution, coverage map, incidence rates, satisfaction) are computed from the incident log". This brief covers what the observer shows, and what the kernel must expose, for:

- incidents (an outbreak, a fire, a flood or storm, a raid or siege), one at a time with their causes, and as aggregates;
- coverage for services that exist or are planned: water, the watch's rounds, help at a fire, a healer;
- tracing an outbreak or a fire to its cause from the kernel's records, beside what people in the world know;
- the five hazard god tools: how they are offered and recorded;
- the chronicle and the "why" for all of these.

The sibling briefs (`m6-health.md`, `m6-fire.md`, `m6-war.md`) own the mechanisms. This brief names what the observer needs from each, not how it happens.

**What exists to build on** (checked in the code):

- **Takings, in layers.** The Takings panel (`web/src/order.ts`; `renderOrder` in `web/src/ui.ts`) shows each taking as "What happened", "What people believe" and "What the gathering was told". The layers come from separate wire tables joined only by the incident's number (`IncidentLine` and `KnownLine` in `Order`, built by `kernel/crates/civ-sim/src/frames/order.rs`; ADR-0015 §1, §7). The frame carries totals over every incident and the newest 60 (`MOST_INCIDENTS`). Every M6 incident kind can follow this pattern.
- **The chronicle.** `ChronicleEvent` (`kernel/crates/civ-agents/src/history.rs`) keeps a kind, people, a settlement, a place, a number, a name and a firm, and is rendered to `Span`s: text, person, settlement, workshop. The wire's `ChronicleEntry` carries only `seq`, `minute` and `spans`, so the place is dropped, and no entry can name another as its cause. A taking enters the chronicle only when someone saw it (`population/crime.rs`). A building's failure is told with what gave way, "under what and why" (`ChronicleKind::BuildingFailed`).
- **Causes of death.** `Cause` has unspecified ("illness or accident"), starvation, childbirth, collapse, violence and a fall from a crossing. Codes are append-only.
- **Receipts and law histories.** Each choice keeps a receipt (wire `Decision`: chosen, runner-up, others, exclusions with reasons), shown in the inspector's Why block. Each law keeps its sponsor, every stance with its reason, and its decision (the Government panel).
- **God tools.** Eight god-tool commands sit in `CommandBody` (`SpawnFamily`, `IntroduceTechnique`, `PlaceDeposit`, `Whisper`, `TellOfIdeology`, `SendAgitator`, `Bless`, `SendWave`), mirrored by `Request` in `kernel/crates/civ-host/src/protocol.rs`. The M4c and M5 tools make `Influence` records (`civ-agents/src/influence.rs`, kinds 0–5): a repeat refreshes one and never stacks, a blessing counts the draws it turned, and the inspector's "observer's hand" lists each as "one recorded influence". Map tools are armed from buttons in `web/index.html` (family, deposit, agitator, wave). No hazard tool exists.
- **Weather.** The clock carries today's weather (`DayWeather`) and the Weather panel the months (`WeatherReport`). One storm a month is a keyed draw on one day, the same for every roof (plan §9, slice U). The map tints the land by season and whitens it above the snow line (`setSeason` in `web/src/map/view.ts`). Rain, wind and high water are not drawn.
- **The map** draws terrain, rivers, people, settlements, fields, buildings, worn ground, crossings, deposits and earthworks. It has one toggle (rivers), and no thematic overlay or legend.
- **Selection** (`Selection` in `web/src/state.ts`) is a person. Workshops have a page; buildings and crossings have only the pointer readout.
- **The watch** keeps counts (`WatchRecord` in `civ-agents/src/polity.rs`: rounds, minutes, cases, and the household its next round starts at). Where its rounds went is not kept.
- **Fetching.** The snapshot's revision counters (`fields_rev` to `crossings_rev`, id 19) prompt queries; the takings and the government are fetched once a day (`web/src/main.ts`). ADR-0003's revisit trigger is per-person snapshot entries past about 64 KB, so ADR-0016 §7 keeps per-person state in queries.
- **The dashboard** (`kernel/crates/civ-host/src/dashboard.rs`) has an Epidemics row, grey: "no disease until M6". Its crime row shows recorded takings beside actual ones, ungraded (12-04 §4.B).

## Where the reports agree

Seven points recur across 15-01 to 15-05, 12-07 and 12-04. I treat them as invariants.

1. **Coverage is measured, never a radius.** "Treat coverage as a measured probability—not a radius attached to a building" (12-07, executive recommendation). Policing is not "a building that reduces crime within a radius" (12-04, executive conclusion), and "a patrol should affect places it actually visits" (12-04 §1.4). A food lens keeps food that exists nearby, can be reached, is permitted and can be afforded apart; one green coverage radius "would hide the interesting simulation" (15-01 §3.1).
2. **What happened, what was reported, what an institution knows and what people believe stay apart.** A fire can exist without a report, and a report without a fire (12-07 §1.2). A dispatcher acts only on what reached its institution, while "the player's diagnostic interface can know more than the institution does" (12-07 §1.3). 12-04 §5.2 calls separate truth and knowledge layers "the highest-value architectural decision". A receipt keeps an input as believed, and the observer should compare it with the world (15-03 §1.4).
3. **An incident is a chain of stamped steps.** Occurrence, detection, notification, dispatch, departure, arrival, useful action, sufficient response, resolution and release are separate times, and an arrival alone never decides success (12-07 §1.1). 12-04 §1.4 splits response the same way. A collapse or a failed harvest needs "mechanical explanations, not fictitious intentions" (15-03 §1.5).
4. **The kernel records explanations, and nearness in time is not a cause.** The critical decision is to "record explanations in the simulation kernel" (15-01, executive recommendation). "Temporal proximity alone should not create a causal arrow" (15-01 §3.5). `caused_by`, `enabled_by`, `responded_to`, `part_of` and `occurred_after` "are not interchangeable" (15-02 §1.1). In a chart, "chronological proximity alone must not become a 'why' statement" (15-04 §1).
5. **An aggregate shows its scope, its denominator and those not served.** Scope must be visible, and the metric must match the question: "taxes assessed are not taxes collected" (15-04 §2). Flows show their period length and rate denominator (15-04 §1). Never receiving help is its own outcome, or "dropping unserved incidents can make a failing service appear faster" (12-07 §1.8); unserved and late incidents stay visible (12-07 §6.6). Small settlements give unstable yearly figures (12-04 §4.H).
6. **The observer selects what to show; nothing selects what happens.** Add "an attention director that selects existing events, not a story director" (15-01 §2, on RimWorld); an observer system "should usually select what to show, not what must happen" (15-05 §2). Notifications run from chronicle-only to an opted-in pause, grouped by episode (15-01 §4).
7. **A god tool makes a stated physical change, reliably, and is recorded.** Tools submit only physical changes and perceptible events (15-05 §4.2). A miracle "should not randomly fail after the interface has promised a definite physical result" (15-05 §4.1). "One recorded influence" is not "caused the entire outcome" (15-05 §6).

**What games teach to avoid:**

- Red and green as goals: Victoria 3 removed them from market balances because players made everything green (15-01 §2). Show direction and magnitude, not desirability.
- Passive hazard reduction around a station (Cities: Skylines II): replace it with "actual inspections, training, maintenance, or behavior changes" (12-07 §6.4).
- A storyteller that deals raids and storms into the story (RimWorld, 15-01 §2), and threats that grow with wealth (15-05 §2).
- Help that turns into punishment: Reus's greed (15-05 §2); "a prosperous town need not become aggressive" (15-05 §4.6).
- Tools that write outcomes: WorldBox's Spite made a kingdom wage total war (15-05 §2).
- Unbounded notifications, which Victoria 3 cut by about half by changing selection rules (15-01 §4); and tooltip-only explanations (15-04 §7).

**Where they differ, or differ from the plan:**

- **Logged or probed coverage.** Plan §5.4 computes the coverage map "from the incident log". 12-07 §1.8 separates potential access, operational coverage and realized service, and §6.2 computes maps from "dry" dispatch probes on copied world states. The log gives only realized service.
- **Coverage as an input.** In plan §5.4 each rung of a service's ladder "sets response speed, suppression rate and coverage". In 12-07 coverage is an output of the response chain, and §5 warns against "a universal time threshold". I read a rung as a capability (equipment, roles), with coverage measured.
- **Satisfaction.** Plan §5.4 lists it among the dashboards. None of these reports gives a measure, and 15-01 §2 warns against desirability judgements. ADR-0016's grievances held against an office are the nearest recorded thing.
- **Clustering on a source.** Plan §4.7 asks that "waterborne epidemics cluster on contaminated sources". 12-04 §4.F warns that sparse counts alone can make concentration look dramatic, and asks for a randomized baseline.
- **Limits on god tools.** 15-05 §5 tests 4, 12 or unlimited interventions per 30 minutes, and separate settings for "destructive power access". The plan sets neither.
- **A chart engine.** 15-04 recommends Apache ECharts as the one general chart engine (§3). The observer has none: its only dependencies are FlatBuffers and PixiJS (`web/package.json`), and the market panel draws SVG sparklines.

## 1. What the observer should show

### 1.1 Incidents, one at a time

**Reports.**

- An incident holds its actual location and severity, occurrence, evolving condition, required capabilities, progress and resolution. A report holds its sender, recipient and perceived location and severity. A mission moves from "assembling" through "on_scene" to "available" (12-07 §6.1).
- Keep several response measures: the first person to arrive, the first water applied, the first trained responder (12-07 §1.1). Shared shocks such as storms and conflagrations strike many hazards together (12-07 §1.2), and disasters cause joint failures of staffing, water and roads (12-07 §5).
- Shared causes are shared records: a drought "should not be copied into thousands of full narrative chains" (15-03 §1.4).
- Use intervals where precision is unavailable, such as "during the spring planting period" (15-02 §1.1).

**My proposal.**

- **One Incidents panel: the Takings panel, generalized.** Per settlement, newest first, filtered by kind: takings, cases of illness, fires, high water, storm damage, raids. Each row keeps ADR-0015's three layers, labelled: *what happened* (the kernel's truth, which nobody in the world reads), *what people know* (how many, from how many first-hand accounts) and *what the polity knows and did*.
- **An incident page,** reached from the panel, the map, the chronicle or a person. Its first lines say what, where, when, whom it touched and how it ended. A timeline of stamped steps follows: began, noticed (by whom), told (to whom), help came (who, from where, when), useful work began, ended, and the losses. A step never reached reads "not reached"; one the kernel did not record reads "not recorded" (15-02 §1.9; 15-04 §1). A step known only to the day is given as a day, never an invented hour.
- **Episodes.** An outbreak, a conflagration, a flood or a siege is an episode record whose incidents are `part_of` it. Its page shows counts, a paged list of members and the shared cause once (the river's peak, the storm's gusts).
- **What each kind must name** (the mechanism briefs supply it):
  - *a fire:* its ignition and source (a hearth, a workshop, lightning, a person, or the observer); each building it reached, from which, and why (distance, material, wind, dryness); who fought it with water from where; what was lost;
  - *a case of illness:* the person, the onset, the exposures the kernel recorded (water drawn, company kept), whether anyone recognized it, and the outcome;
  - *high water, a storm, a drought:* its footprint and magnitude in the weather's units, and what it touched (fields, buildings, crossings), linked to each failure it caused;
  - *a raid or a siege:* who came, from which polity, under which decision; what was taken or burnt; every blow, told as M4c's encounters are; and the ending, with both sides' law histories.
- **On the map,** the period's incidents are marks at their true places, drawn with more than colour and without flashing (15-04 §6). Selecting one highlights its participants and recorded consequences (15-01 §3.5). The camera moves only on "Locate" (15-04 §1). Rain, snow, wind and flood water are drawn from kernel fields, never guessed by the page.

### 1.2 Aggregates: rates, maps and trends

**Reports.**

- Flows are period bars or rate lines that show "period length and rate denominator"; distributions keep affected counts (15-04 §1). Use extrema-preserving summaries "where brief famine, price, mortality, or storage spikes matter", and keep missing observations as gaps (15-04 §4).
- Recorded crime can rise while safety improves (12-04 §4.B), and better reporting "can make recorded performance look worse" (12-07 §5). Demand-weighted, population-weighted and harm-weighted performance answer different questions (12-07 §5).
- Keep "a separate player-facing account of harm so that favorable official statistics do not erase victims" (12-04 §5.5).
- Normalize impact "at multiple scales" (15-02 §1.3).

**My proposal.**

- **The kernel computes every aggregate from incident records** (plan §5.4), as a derived view that nothing in the world reads. The page only draws it.
- **Counts before rates.** Each figure states its scope, period, numerator and denominator ("4 fell ill of 212 people this year"). A rate appears only beside its counts: in a village of 300 people each case moves a rate per 1,000 by 3.3 (my arithmetic).
- **Truth beside record.** Where the polity's record differs from the kernel's (illness nobody named, a fire nobody reported), both are shown and labelled, as the dashboard's crime row does.
- **Charts.** Monthly bars of incidents by kind; an outbreak's cases by day of onset; each incident's response times as a strip that includes "none came", never only a mean (12-07 §1.8). Neutral colours, no green or red (15-01 §2).
- **Side by side.** The same measures for each settlement in one table, as plan §2's comparison pillar asks.
- **The observer's share.** Every aggregate says how many of its incidents the observer started ("9 fires, 1 of them the observer's").

### 1.3 Coverage maps

**Reports.**

- Coverage is a distribution for a place, an incident class and a threshold, with never as an outcome. The interface distinguishes potential access, operational coverage and realized service; "a building near a station can have excellent potential access and poor operational coverage when its crew is occupied" (12-07 §1.8).
- Compute maps by dry dispatch probes that "must not reserve resources in the live world", and "show uncertainty where samples are sparse" (12-07 §6.2).
- Coverage is "direction-dependent and discontinuous at barriers", and nominal staffing is not readiness (12-07 §5); "a nominal roster does not establish operational capacity" (12-07 §1.4).
- In early farming settlements, help is household assistance, communal equipment and assembly obligations; layout, daytime field work, water access and cooperation set readiness (12-07 §4).
- Show one principal overlay at a time, with title, scope, units, legend, time basis and population denominator, on fixed scales; draw "unknown", "not applicable" and "zero" differently (15-01 §3.1).
- Opening an inspector must not "consume simulation randomness" or change scheduling (15-03 §4.1).

**My proposal.**

- **A lens control on the map,** one lens at a time, read per home (a household's plot), each with its legend, scope, period and denominator: *Water*, *Help at a fire*, *The watch* and *A healer*. A service that does not exist says so ("No healer lives here", as 15-01 §3.2 shows "No court exists here") rather than painting the map empty.
- **Three layers per lens, named as 12-07 §1.8 names them:**
  - *Could reach:* walking minutes from each home to the nearest source or provider on the walking grid, so a river with no crossing shows as the barrier it is;
  - *Would come:* the kernel's dry probes from where people are at sampled hours (asleep, in the fields, away), as the share of probes in which enough able people arrive within the threshold, with the number of probes shown;
  - *Did come:* realized help at each incident of the period, from the incident records.
- **Water** shows, per home, the sources its household drew from in the period and the walk, as recorded, with access kept apart from permission (M5c's claims). A source's condition is truth and is drawn as such; what the household believes of it is a separate layer.
- **The watch** shows the nights out of the last 30 that a watcher stood at each home. The kernel must keep each home's last stands; `WatchRecord` keeps only counts.
- **Help at a fire,** in a village without a brigade, is neighbours with buckets: the probe counts adults awake and at home within a few minutes' walk, and water within reach (12-07 §4).
- **Probes run in the kernel** on a read-only view; they draw nothing and reserve nobody, and turning them off leaves every digest unchanged. One probe per home a month in a village of 300 homes is 10 route searches a day (my arithmetic), against the 250–800 route-search misses a day measured at 1,000 people (plan §9, slice X).

### 1.4 Tracing a cause

**Reports.**

- Accounting, decision, historical provenance and counterfactual are four "whys" and "different computational products" (15-03 §1.1).
- Edge types include `used`, `believed`, `enabled`, `blocked` and `generated`, and "the distinction between used and caused matters"; showing "missing knowledge and mistaken assumptions" reconciles the actor's model with the observer's (15-03 §1.4).
- A lasting property keeps a provenance certificate; a stone house keeps its material choice, procurement and construction apart (15-03 §1.5).
- Recognize patterns such as "recurring problem → proposal → political decision → implementation", never script them; "require recorded response links where the text will claim a response", and judge a condition when its event occurs (15-02 §1.4).
- Several witnesses repeating one rumour are not several observations (12-04 §1.5). Missing evidence gives "a visible limitation, never an invented reason" (15-03 §4.5).

**My proposal.**

- **Typed links, recorded when they happen.** Incident and episode records carry `caused_by` (an exposure to a case; an ignition or a burning neighbour to a fire; a storm to a failure; the observer's influence to whatever it started), `part_of` (a case to its outbreak) and `responded_to` (a proposal to the incidents its sponsor's issue names). Text says "after" where only time joins two things.
- **Two columns on every incident and episode page:** *what the kernel recorded* and *what people in the world know*, each with its sources. No choice ever reads the first (ADR-0015 §1).
- **The cholera outbreak.** The outbreak's page lists each water source with the cases among those who drew from it and among those who did not, as counts. The truth column says what fouled the source and since when. The knowledge column says who suspects the well and on whose word, who stopped drawing from it, and any law proposed about it, with its stances. If nobody in the world links the illness to the well, the page says so: the trace is then the observer's alone.
- **The great fire.** The episode page draws the spread from house to house, each step with its reason. "What followed" lists the laws whose issue names the fire, with each stance's reason (a household that lost its home; one that cannot afford stone). A house built in stone under such a law keeps the law in its provenance, as plan §2's "building code after the Great Fire of year 41" asks.
- **The siege.** Its page keeps each day's supply, who left, who struck whom, and the decision that ended it, with the annexation's law history on both sides, as M5c shows an agreement.
- **Why not.** "Why did the fire not reach this house?" is answered only where the spread model recorded the house as weighed, with its chance and draw. A house never weighed reads "not weighed", never a reason found afterwards (15-03 §1.2: "recomputed alternative, not an option considered at the time").

### 1.5 God tools for hazards

**Reports.**

- Whisper, omen and miracle are channels with different promises, and a miracle delivers its stated physical effect (15-05 §4.1). No tool may reach operations like `declare_war` or `pass_law` (15-05 §4.2). Keep one active record per recipient, proposition and episode (15-05 §4.5).
- Before use, show the certain direct effect, the current situation, plausible responses and unknowns, and no exact chance of an outcome (15-05 §6).
- Magnitude has reach, intensity, duration, irreversibility and propagation; charging for predicted impact "would imply foresight the simulation does not possess"; speed must not change susceptibility (15-05 §5).
- The chronicle should "distinguish direct effects, explicit agent attribution, and broader inferred consequences" (15-05 §6). Ordinary maintenance should need zero god actions (15-05 §4.7).

**My proposal.**

- **Five map tools,** armed as the wave and the agitator are, each a physical change with a footprint, a magnitude and a time, within ranges as `Bless` has them:
  - *Fire:* an ignition in one building, at a chosen hour.
  - *Plague:* one person falls ill of a chosen disease, as if infected elsewhere. Whether it spreads is the disease model's business.
  - *Flood:* a reach of river runs higher by a stated amount for stated days.
  - *Drought:* rain over the landscape falls to a stated share for stated months.
  - *Storm:* a storm of stated gusts over an area on a chosen day, beside the world's own monthly storm, whose keyed draw it leaves alone.
- **The preview** states only the certain effect ("a fire starts in Ada's longhouse at 14:00"), what the kernel knows of the situation (who is home, today's wind, the nearest water), and that what follows is people's. It gives no forecast of losses.
- **Records.** Each use is an `Influence` record of a kind appended after the wave. Every incident it starts carries the record's number as `caused_by`. Panels, aggregates and the chronicle mark it "the observer's", and the chronicle says "one recorded influence". A repeat on the same target in the same period refreshes the record.
- **People never learn that the observer did it.** Omens and attribution (15-05 §4.4) are not M6's.
- **Not offered:** a tool that starts a war, a raid or a siege, plants a belief (a well believed fouled), or passes or blocks a law (15-05 §1, §4.2).

### 1.6 The chronicle and "why"

**Reports.**

- Detect processes as well as incidents, with scope, interval and baseline, and use hysteresis against repeated "decline" and "recovery" (15-02 §1.2).
- Normalize impact at several scales, select a diverse set, and keep room for everyday life: "a famine should not occupy every card" (15-02 §1.3). Templates aggregate ("Three harvests failed in five years", 15-02 §1.7); an unsupported account invents prosperity, emotion and rationale (15-02 §1.6).
- "A mass migration or epidemic may create many overlapping candidate stories" (15-02 §3.3), and "a disaster must not erase every quiet citizen's latest explanation" (15-03 §3.3).
- Explain at four levels: a causal sentence, a contrastive breakdown, provenance, a counterfactual (15-03 §4.2).

**My proposal.**

- **Told by episode.** One entry when an outbreak, a conflagration, a flood or a siege begins, and one when it ends, with counts ("14 fell ill at Ashford between 3 and 20 June; 5 died"). Its members are reached from the episode's page. Small fires put out at once are summarized monthly, as ADR-0015 §7 summarizes minor takings. Every death is told, and resolves to an incident and a `Cause` (appended: illness by name, fire, drowning).
- **Plain words:** who did what to whom, under which law or office (ADR-0015 §7), and nothing graphic (the M4 observer notes, `m4-observer.md`). Where truth and knowledge differ, the entry says which it tells ("nobody at Ashford knew the well was fouled").
- **Links in the text.** New span kinds for an incident and a place let an entry open its page or locate it on the map.
- **Attention.** New kinds are chronicle-only by default; the observer may opt in to a toast or a pause by kind (15-01 §4). Pause stays off by default, and the camera never moves by itself (the M4 observer notes).
- **"Why" on an incident page,** two levels in M6: a causal sentence from the recorded links ("caught from Bram's longhouse, 6 m upwind; dry thatch"), then the chain of provenance. Choices made in a hazard (who ran to the fire, who fled, who stopped drawing water) keep receipts with appended reasons in the existing Why block. The counterfactual level is deferred.

## 2. What the kernel must expose

Every addition appends to `tce_wire.fbs`: new field ids, union members and enum values at the end, each a minor bump (ADR-0001). Truth and knowledge stay in separate tables joined by number (ADR-0015 §7), and the kernel renders the words. Names are suggestions.

| Addition | Carries | Fed by |
|---|---|---|
| `Snapshot.incidents_rev` (id 20) | Changes when an incident or episode opens, moves on or ends | Every incident kind |
| `GetIncidents {settlement, kinds, after, limit}` → `Incidents` | Rows newest first with a cursor; totals by kind over all time, recorded beside actual | Incident records |
| `GetIncident {id}` → `IncidentInfo` | Stamped steps; typed links by number; truth lines; knowledge lines with sources; the polity's lines; an influence number | Incident, belief and case records |
| `GetEpisode {id, after, limit}` → `EpisodeInfo` | Kind, span, footprint, shared cause, counts, paged members, ending | Episode records |
| `GetSeries {settlement, kind, months}` → `Series` | Monthly counts with numerator, denominator and the observer's share | Derived; nothing reads it |
| `GetCoverage {settlement, lens, layer}` → `Coverage` | Per home: minutes, a share with its probe count, or realized times; "unknown" and "not applicable" flagged | Walking grid, probes, incidents |
| `StartFire`, `SeedIllness`, `RaiseRiver`, `WithholdRain`, `SendStorm` in `CommandBody` | Target, magnitude, start and duration, as `Bless` and `SendWave` carry theirs | Influence records |
| `SpanKind` `Incident` and `Place`; `ChronicleEntry` `place` and `incident` | Links from chronicle text | Chronicle events |
| `InfluenceLine.kind` codes 6–10; a person's illnesses and exposures in `PersonInfo`; `BuildingState` `Burnt`; flood extent | The observer's hand, the inspector, the map | Influence, health, fire and weather records |

- **Not in the snapshot.** An outbreak can have hundreds of cases. Incident state stays in queries, paged by cursor, since historical events "must not disappear because the panel could not keep up" (15-04 §4).
- **Saves** keep incidents, episodes, links, exposures and hazard influences. Aggregates and coverage are derived and rebuilt on load (AGENTS.md).
- **ADRs.** The incident record, its layers and its links generalize ADR-0015, so I suggest amending it rather than adding an ADR, and keeping the hazard tools under ADR-0016 §5. That leaves plan §1's three ADRs to the mechanism briefs.

## 3. What to defer, and why

| Defer | To | Why |
|---|---|---|
| The counterfactual level of "why" (15-03 §4.2) | Later | It needs sandboxed reruns; branching from a save (plan §2) serves meanwhile |
| Story patterns beyond episodes (15-02 §1.4) | Later | Episodes carry the demo's chains; a pattern language needs its own tooling |
| Narration by a language model (15-02 §1.8) | Not planned | Plan §9 plans no LLM spend, and templates are "a complete shipping path" (15-02 §4) |
| A historical map with a date slider (15-02 §1.9) | M8 or later | It needs past states kept, which 15-01 §3.5 separates from inspection |
| Omens and divine attribution (15-05 §4.1, §4.4) | Later | Not in plan §2's god tools |
| Satisfaction as a score | Not planned | No report gives a measure; grievances against a provider stand in |
| Accessibility checks in the Unreal host (15-04 §6) | Unreal work | They must run in the packaged host |
| Earthquake; changing the climate | M8; M9 | Plan §2 |

## 4. Risks and validation

- **Leaking truth.** No choice may read an incident's truth, a source's condition, a probe or an aggregate. Test it as labels are tested: digests with aggregates and probes on and off are identical (the M5 relations brief §1.1).
- **Plot.** No hazard grows with prosperity or with time since the last one (15-05 §2). The demo's outbreak and fire come from lived worlds, or from a god tool recorded as such. If no lived world shows an outbreak traced to a well after a week's tuning, show it in a test world, as M5 did with the treaty, and log a `NUDGE:` (plan §1, rule 3).
- **Floods of records.** An epidemic or a conflagration writes many incidents at once (15-02 §3.3), and 15-05 §7 asks for benchmarks of "mass-witness events". Episodes, paging and a cap per response keep payloads small; time a conflagration at Max.
- **Small numbers.** Counts come first, and no rate is graded on one village's year (12-04 §4.H).
- **Dark content.** Deaths by fire, illness and war are told plainly, actor and institution named, and the camera never moves by itself.
- **Too many panels.** One Incidents panel and one lens control, not a panel per service: "build one complete investigation before a broad interface" (15-01 §6).

**Kernel tests.**

- Probes draw nothing: a world lived with the Coverage query called every day matches one lived without it.
- Every death in a hazard resolves to an incident and a `Cause`, and every link points at a record that exists after save → load → save, which stays exact.
- Aggregates reconcile with the incident records they count.
- No hidden command path (15-05 §7): a hazard command writes only its physical change and its record.
- Paired runs: the same fire set with the neighbours at home and in the fields differs in the help that came (15-05 §7; 12-07 §6.6). A storm sent leaves the world's own storm series unchanged.

**Browser tests,** each from a world a civ-sim example makes, as `theft_world` makes the M4b demo's (`web/e2e/host.ts`):

1. The Incidents panel lists as many rows as the state holds, each with its three labelled layers; an incident page lists its steps in time order, and a step not reached reads "not reached".
2. One lens at a time, each legend naming scope, period and denominator; "No healer lives here" in a world without one; a home across a river with no crossing reads a longer walk than a farther home on the near bank (12-07 §5).
3. The fire tool on a building: one chronicle entry saying "one recorded influence"; the incident marked "the observer's" in the panel and in the aggregate; a second use on the same building adds no record.
4. An incident link in the chronicle opens its page, and the map moves only on "Locate".
5. Each new panel shows its empty, loading and error states (plan §1, rule 2), and an incident page reads the same after save and load.
6. `m6-demo.spec.ts`, run with `TCE_DEMO=1` as the earlier demos are: each part from a saved world, showing whatever happened.

## 5. Open questions for the designer

1. **One panel.** Fold the Takings panel into one Incidents panel, or keep it beside a new one?
2. **Which coverage layers ship.** "Could reach" and "Did come" only in M6, with probes later?
3. **What the chronicle tells.** Truth nobody in the world knows (a fouled well), or only what someone saw, as with takings?
4. **The plague tool's target.** A person only, or also a water source? A source makes "traced to a well" the observer's doing.
5. **Satisfaction.** Shown as grievances held against a provider, or dropped from plan §5.4?
6. **Destructive tools.** Behind a setting chosen at world creation (15-05 §5), or always offered?
7. **The ADR.** Amend ADR-0015 for incidents, episodes and links, or write a new one?
8. **Charts.** Adopt one chart engine now (15-04 §3 recommends ECharts), or keep drawing SVG as the market panel does?
9. **Flood water on the map.** A kernel raster of depth by day, or the river's reaches drawn wider?
10. **The siege page.** Is it this brief's, or `m6-war.md`'s?
