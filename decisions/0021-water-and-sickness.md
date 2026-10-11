# ADR-0021: Water and sickness: sources people choose, loads that are conserved, infections no choice reads

Status: Accepted
Date: 2026-10-10
Milestone: M6a
Amended: 2026-10-10, M6a slice AY step one, to match what was built: §2's heads are settled to
where the mean recharge holds them, not spun up over years (plan §9). And M6a slice AY step
three: §3's well is a record of its own, as a crossing is, not an earthwork with a building's
groups: its shaft dug at ADR-0010's rate, its one lining rotting and drawn at a quality as
ADR-0009's parts are; its household weighs it over the lining's life, but each session of the
work weighs a year's saving (plan §9). And M6a slice AZ step three: §5 gains when an outbreak
opens and closes, the rule ADR-0015 §8 leaves to this ADR.

## Context

Through M5 water is a walk to the nearest river or lake cell (`nearest_water`,
`Behavior::FetchWater`), at a fixed 20 L a person a day. There is no groundwater, spring or well:
each river reach keeps one mean annual discharge. Waste is a household's midden, and nothing
records where anyone relieves themselves. Deaths come from a Siler life table and hunger as
competing risks; `Cause` has no disease, and the README says disease is "not modelled apart from
the life table".

M6's demo includes "a cholera outbreak traced to a well" (plan §7), and plan §1 asks that it come
from physical processes and people's choices, never a script. The M6 health brief
(`docs/briefs/m6-health.md`) digests 12-01, 12-02, 05-03, 05-04, 05-05 and 03-02. Its invariants
are that water is a chain and not a bonus, that no infection arises without a source, that
exposure follows what people drink and share, that people act on what they observe, that
infection, symptoms, infectiousness and death are separate, that deaths are not counted twice,
that water and waste are conserved, and that a law acts only through work and behaviour.

What makes this expensive to reverse: new saved state (groundwater, wells, loads, infections,
what people saw and suspect), new content kinds, an appended cause of death, appended claim,
issue, policy and influence kinds, and the save schema.

## Decision

### 1. Water is a chain from a source people choose

- **Sources** are river and lake points, springs and wells. A household chooses among the
  sources its people know by the walk, the wait and lift at the source, and its own suspicion of
  it (12-01 §1.5). Sources are places in `uses.rs`, so households log where they draw and whom
  they saw there.
- **Use flexes with access.** The fixed 20 L becomes a target that falls as a trip costs more,
  within 12-01 §2.1's 10–30 L prior for carried water (a tuning curve).
- **Rights, v0.** A household's well serves its own people and its kin's households; a
  polity's well serves its members. A household whose people are short of water may draw at any
  source it knows (need over another's word, as for claims, plan §9).
- **Rivers by the day.** One runoff store per world, fed by the weather's daily rain and melt
  and drained by a recession, scales each reach's mean discharge to the day's flow (a
  simplification of 03-02 §1.1). It sets dilution, low water in droughts and, from M6b, floods.
  Walking stays on mean discharge in M6a: who can ford where is not changed until M6b designs
  floods.

### 2. Groundwater is saved; springs are derived

- **A shallow aquifer** keeps a head per coarse cell (03-02 §5.1's 64–128 m). Its conductivity
  and specific yield are drawn by seed within 03-02 §2.2's priors by landform. Recharge is what
  the weather's reference soil cannot hold (until now discarded). Cells exchange with their
  neighbours and with river cells, and wells draw on them. "Do not give each well an independent
  renewable supply" (03-02, executive recommendation).
- **Heads are settled** to where the landscape's mean recharge would hold them for good, when a
  world is made (before the year a new world lives first) or when a save from before this ADR
  loads: what years of its climatology average toward, at a fraction of the cost.
- **Springs** are derived, never saved: a land cell off the river network where the head stands
  above the ground, flowing what the aquifer sheds there, dry when heads fall.

### 3. Wells are pits with parts (amends ADR-0009 and ADR-0010)

- **A well is an ADR-0010 pit with ADR-0009 lining parts** that rot by wetness, a cover and a
  curb. Its yield is its water column, refilled by inflow from its cell's head (03-02 §1.4); a
  draw takes from the column, and an empty one means waiting or another source.
- **A household digs one as it weighs a log footbridge** (ADR-0009 §9): once a year, the walking
  and waiting its fetching would save over a well's life against the hours to dig and line it,
  at the depth it expects from what its people know (wells they saw dug, pits that met water,
  their height above the river they draw from). A shaft meets water where the true head is, or
  not; a dry shaft is deepened or given up, and its depth becomes knowledge. Digging is 8 h/m³
  (ADR-0010 §2).
- **Keeping it** is chosen work: mending the lining, clearing silt, making a cover and curb.
  A polity's well is built together by law, as a crossing is.

### 4. Waste and contamination are conserved loads

- **A load per disease** is kept in each midden, well, river reach and household's stored water.
  An infected person's shedding goes to their household's midden when at or near home, and to
  the ground where they work otherwise; a heap's size creates nothing.
- **Four routes move loads, each move recorded:** runoff from middens into wellheads and banks
  just downslope on a day of heavy rain (far less into a covered, curbed well); sparse
  groundwater links downgradient with a travel time and decay (03-02 §1.6; 12-02 §5.3); river
  transport reach by reach with the day's flow, diluted and decaying (05-04 §5.2); and mixing
  in a household's stored water. Concentration is load over litres. Decay and runoff shares are
  tuning values, logged as such.
- **Nothing creates a pathogen.** "Do not spontaneously generate a particular infection merely
  because a midden becomes large" (12-02 §1.1).

### 5. Diseases are content; infections are truth

- **A `disease` content kind** with its own routes, its three clocks as distributions, the share
  with symptoms, severity by age, shedding by stage, decay outside the body, immunity, and death
  hazards by stage (05-03 §1.1, §1.3, §7.3, §7.6). Cholera and bacillary dysentery come first.
- **An episode per infection** keeps its stages and their end days, immunity until a day, and an
  acquisition record: the day, the route (water, household, care, the observer) and the source.
  Exposure is summed over routes each day and drawn once (05-03 §7.2).
- **The ill are kept from work** as the hurt are (`Facts::hurt`). **Tending** is chosen work:
  it costs hours, exposes the carer, and on a severe day lowers the death hazard by the
  template's supportive effect (05-05 §2.2's 0.95 prior for enteric disease). Fluid replacement
  is a technique to find or be given, which brings cholera's severe hazard toward 05-03 §3.2's
  treated figure.
- **Deaths** are `Cause::Disease` (appended), the episode naming its disease. Until a disease is
  measured endemic in lived worlds, its deaths are excess over the Siler baseline (05-01 §1.3).
- **Outbreaks** (amended, step three; ADR-0015 §8 leaves the rule here): an outbreak is the cases
  of one disease among the people of one settlement. A case taken while one is open there is
  `part_of` it; one taken while none is begins one, and its acquisition record is the outbreak's
  cause. It ends at the first midnight when none of its cases has run for as long as a new
  infection of the disease can take to show (its incubation's mean and three spreads, at least a
  day). Saved with its cases' links; the chronicle tells it when it begins and when it ends, with
  its counts.
- **No choice reads an episode's acquisition record, a source's load, or anyone's infection**
  beyond what is seen.

### 6. What people know: sickness seen, suspicion tallied

- **Seen.** The household and anyone who enters the home see someone abed; the household and its
  kin know a death. Seeing makes an appended claim kind, a household had sickness from a day,
  told as claims are (ADR-0016 §3).
- **Who draws where.** Each household logs its own draws and the households it saw drawing at
  the same source that day; a person also knows the usual source of those they have ties with
  (a design prior).
- **Suspicion is per person, a tally of their own records:** of the households they know that
  draw at a source, how many had sickness lately, against those that draw elsewhere, above a
  ratio and a minimum count (design priors; Snow's contrast, 12-02 §4, orients and never
  targets). A suspicion is told as an appended claim kind with its counts; one origin's
  retellings count once.
- **Water only, v0.** Tallies of contact with the sick, and quarantine, wait until a lived
  outbreak shows what people would need them for.

### 7. Responses go through existing pipelines

- **Households** avoid a source they suspect when they know another, cover their own well, tend
  their sick, and reconsider where to live after a death or a suspicion (ADR-0018 §5).
- **Polities.** Appended issues (`sickness`, `source_suspected`, `water_far`) open moves and
  weigh toward none (ADR-0013 §5). Appended policies: *close a source*, kept by each person's
  choice as a curfew is, and *dig a well*, built together as a crossing is, an equal share of
  work asked and nothing following a share not given. Each household's stance is its own
  forecast in today's units: hours walked or worked against days of work it believes the
  sickness costs, a death counted as `w_death` days (a design prior).

### 8. Introductions are the observer's

- **No disease without an introduction.** Until imports from off the map exist (M9), the
  observer's plague tool is the only one: a named person falls ill of an authored disease, as if
  infected elsewhere. Whether it spreads is the disease's and people's. A source as the target
  is not offered: it would make the source the observer's choice.
- **The drought tool** holds the weather's slow anomaly at a dry value for 3–24 months, keeping
  wet-day persistence and temperature coherent.
- Each use is an `Influence` record of an appended kind; every infection and month it touches
  carries its origin, so statistics can leave it out (03-07 §1.3). People remember it as any
  other.

### 9. The boundary

- **Saves** gain the aquifer's heads, wells, loads, episodes, sickness and suspicion claims, and
  source logs; the schema is bumped. A save from before loads with no wells, no disease, no
  loads, and heads spun up.
- **Content** gains the `disease` kind, a well-digging technique (founders bring it: timber-lined
  wells are documented in the early Neolithic, 12-01 §3.1), activities, issues and policies.
- **The wire** shows sources, wells and water columns on the map, a person's episodes and
  suspicions in the inspector, and illness as incidents and episodes by query (ADR-0015 §8).

## Consequences

- Easier: an outbreak traced to a well is explained by who drew where, who saw whom abed and
  what each person tallied; M6b's floods and M6c's sieges find water already a chain with real
  sources.
- Harder: water and sickness add daily work per household, source and link; Gate B (ADR-0011
  §5) must cover cases, sources and water use, with wide tolerances. A world with no
  introduction changes only by its water choices and groundwater.
- Forbidden: an infection without a source; a disease spawned by a midden's size, a flood, a
  famine or a date; any choice reading an acquisition record, a load or another's infection; a
  mortality bonus for healers or water works; an issue weighted toward a policy; a well that
  creates water.

## Alternatives considered

- **SEIR per disease template** (plan §5.4): one generic system with different settings, which
  05-03 rejects (executive recommendation, §1.3).
- **A contamination radius:** distance is a heuristic; travel time and connection decide
  (03-02 §1.6; 05-04 §1.1).
- **A ladder of water works** (well, cistern, aqueduct, pipes): 12-01 §6.4 asks for interacting
  capabilities instead.
- **Disease as a constant background:** a closed world loses an acute disease after an outbreak
  (05-03 §4.3); an invisible permanent case would be a script.
- **Suspicion read from the kernel's acquisition records:** it would let people know the truth.

## Revisit when

- Imports from off the map arrive (M9): introductions by travellers, an external population.
- Towns need latrine pits, drains, cisterns or boiling (12-01, 12-02).
- Healers become a calling (M10).
- A disease is measured endemic in lived worlds: refit the Siler residual.
- Goods carry batch identity, making food a route (12-02 §4).
