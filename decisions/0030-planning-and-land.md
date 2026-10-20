# ADR-0030: Planning and land

Status: Accepted
Date: 2026-10-18
Milestone: M8c
Amends: ADR-0004 §3 (a plot given under a plan), ADR-0007 §1 (no sale of land in M8, restated),
ADR-0013 (an appended law kind with saved ground), ADR-0022 §12 (nuisance rules beside the building
rules), ADR-0029 §1 (a reserved corridor becomes a way when first worked), plan §5.1 (height and
the planning institution)

## Context

Plan §7's M8 lists "Planning as politics: planning institution, plan templates, zoning, land
value". Plan §5.1 has governments decide whether a planning institution exists, and has people
choose "Which plot to claim or buy" and "how tall (land value, tech)". The planning brief
(`docs/briefs/m8-planning.md`) reads 10-04, 10-05, 08-19 and 10-08, with 08-11 and 10-07. It finds
eight points they share:

- planning is coordination by an institution, not a flag;
- a plan is proposed by those who gain;
- the ideal, the reservation, the built and the surviving are four things;
- land assembly is its own problem;
- allocation is stored apart from geometry;
- law is not self-enforcing;
- land value is an outcome of bids;
- density and persistence are consequences.

What the kernel has (checked in the code):

- laws with templates, and issues that only open moves;
- laws that keep ground of their own: crossings, wells, banks, enclosures, ways;
- building rules kept by each builder's choice (ADR-0022 §12);
- plots with a household and a use, but no status;
- fields chosen by worth.

Homes, by contrast, are **placed by a geometric rule**. A new household's wanted point lies 20 m
farther from the hearth than its parent's, within 300 s' walk of the hearth
(`found::HOME_REACH_SECONDS`), and `build::home_site` takes the first clear, level plot in rings
around it. No alternative is compared and nothing is paid. The reach caps a settlement at some
28 ha, about 2,800 people at 100 a hectare. No ledger channel pays for ground, and ADR-0007 has no
land market.

## Decision

### 1. Where a household builds is a choice, and its receipt is the only land value

- **A household about to build scores candidate plots in hours a year.** This covers a first home,
  a new home, a store and a workshop. A candidate's hours are:
  - its members' walks from there to the places they go (their fields, the hearth, the water they
    draw, their kin's homes, the doors they buy at), each weighed by the trips the household made
    there, at the walking times the kernel already plans with;
  - plus levelling;
  - plus any fee or ground rent, and an expected sanction (§5), in hours by the household's own
    `hour_kcal`.

  It takes the cheapest.
- **The candidates** come from rings about each such place and about the old rule's wanted point,
  and from parcels the household may ask for (§4). A design prior of 16–32 are weighed a decision
  (10-05 §2.2; 08-19 §2.2), each passing `plot_clear` and the rules the builder keeps.
- **The reach is the household's own reckoning.** `HOME_REACH_SECONDS` no longer bounds where a home
  goes. Candidates are sought out to the walk the household accepts to a field
  (`max_walk_minutes`), so a settlement spreads as far as its people find it worth walking.
- **The receipt is saved with the plot:** the chosen plot, the next best, their hours and the number
  weighed. The hours the chosen saves over the next best are what the plot is worth to that
  household, its reservation value. It is never written on the ground, and no other household reads
  it.
- **Land value is derived, never read.** The observer's land map is drawn from what was paid (the
  ledger, §4) and what was reckoned (receipts), each labelled as such, or "nothing recorded"
  (08-19 §6). A test blanks it and checks the digests.
- **This changes where every world builds.** Its slice records new baselines; every later slice
  keeps a world without a plan on them.

### 2. A plan is a law of one appended kind, and its layout is saved as realised

- **`PolicyKind::Plan` is appended.** Its templates are content: a composition of **operations**,
  with menus a sponsor chooses from. M8's operations:
  - an orthogonal subdivision (block sides and plot modules in metres, corridor widths by rank);
  - a public site (a square at the origin, ground kept for a well, a workshop row);
  - an extent in blocks;
  - an orientation rule (toward the hearth, the river or a cardinal line).

  Further operations (radial, axial, enclosure wards) are appended vocabulary, not a schema
  change. 10-04 §3.4's 40 × 80 m blocks with 6 m streets is a test fixture, not a cultural preset.
- **A pure function in `civ-land` lays a template over the ground as it is:** water, slopes too
  steep, and every claim it touches. It returns corridors (legal width and line), public sites and
  parcels by district.
- **The realised layout is authoritative state,** saved as `BuildingSpec` is (ADR-0004 §2). It does
  not change when claims or terrain change.
- **A sponsor lays 3–8 alternatives:** origins at the hearth, the landing or the built edge, and the
  orientations allowed. It forecasts each as households will and puts forward the best for its
  household and followers.
- **Versions are kept.** An amendment is a new version, by the custom's procedure. A repeal ends the
  reservations and leaves everything built (10-04 §5.1).
- **A plan needs knowledge:** laying out by cord and stake. This is a content technique like any
  other: drawn in founders' repertoires at the content's prevalence, found and spread by people
  (ADR-0008).
- **Three issues are appended.** Each opens a plan move and weighs toward nothing (ADR-0013 §5):
  - *no ground*: a household found no clear plot it would take within its reach, or took one far
    worse than it wanted;
  - *a way built over*: a building begun on a worn trail lengthened walks the household made;
  - *new ground*: households build beyond the settlement's built edge.

  `burned` also opens a plan move, by authoring, as it opens building rules.
- **Each household forecasts a plan in hours,** as `Enclose` does. The forecast counts:
  - the walking it expects to save on a parcel over its next-best candidate, if it means to build
    within the horizon;
  - fees and ground rent it expects to pay or be paid;
  - field ground it would lose under a reservation, at its own `site_value`;
  - its share of the staking work;
  - walks a corridor would lengthen.
- **A sponsor sizes a plan** by the arrivals and new households its members remember.
- **A plan move is weighed only while one of its issues is held,** and at most every 60 days a
  polity (10-04 §3.4 gives 30–90).

### 3. Reservation before construction

- **In force, a plan reserves.** A builder who keeps it treats its corridors and public sites as not
  clear, as a keeper of a gap rule keeps its distance. Buildings already standing stay.
- **Adoption builds nothing.** Staking, clearing a corridor or setting a surface is a public work
  proposed later, in equal shares or paid by `PayWork`, as crossings and banks are.
- **A corridor becomes a way (ADR-0029 §1) when first worked,** with its legal width as the way's
  corridor width. Until then the plan holds it, and routing does not see it.
- **Legal, built and occupied ground are kept apart** per corridor and site, and the observer draws
  them as three layers (10-04 §6.1).

### 4. Land assembly, allotment and payment

- **Every claim a layout touches is listed with a clause.** The template names a clause per kind of
  claim, from a content menu:
  - *around*: the layout bends round the claim;
  - *at the review*: under `LandHolder::Settlement` and `LandUse::Need`, the settlement gives no
    field on reserved ground at its yearly review;
  - *land for land*: the holder is offered vacant ground of like area within the same walk, and
    accepts or refuses by its own `site_value`s;
  - *paid*: the store offers goods the law names per hectare, and the holder accepts or refuses;
  - *taken*: the ground is reserved without consent once the crop is in, and the holder blames the
    gathering (ADR-0016).
- **A standing home is never cleared by a plan**, under any clause.
- **A refusal leaves the component unreserved,** and the gap is recorded.
- **A plan names a keeper:** a new office per plan, with a `Tenure` (ADR-0024). The keeper stakes
  parcels (hours, recorded), opens phases when the last is mostly taken, and allots by the plan's
  rule:
  - *by need*: households short of a home, nearest first;
  - *by lot*: a keyed draw among those who ask;
  - *by offer*: whoever offers most of a named good, paid once or as a yearly ground rent;
  - *by standing*: the keeper's choice, open to regard and bribes through ADR-0026 §4's terms.

  An optional *build by* clause returns an unbuilt parcel at its day.
- **A plot given under a plan records its plan, its parcel and the rule it was given under**
  (amending ADR-0004 §3).
- **No land is sold or leased in M8.** Who may lease or pass on a parcel is unchanged (ADR-0007 §1
  stands), and a parcel stays with its household as any plot does. Land value therefore shows only
  in fees, ground rent, compensation and receipts.
- **Every payment is on the ledger,** on three appended channels:
  - `GroundFee`: an allotment paid once;
  - `GroundRent`: yearly, as an ADR-0015 obligation to the store;
  - `Compensation`: from the store to a holder whose ground is reserved.

  Nothing creates goods.

### 5. Compliance is a builder's choice, and cases are brought, not automatic

- **Keeping a plan is weighed as keeping a building rule is** (norm, stance, regard for the
  sponsor). Where the plan names a remedy, the household also weighs an **expected sanction**: the
  chance it believes that someone sees, that a hearing finds against it and that the remedy is
  carried out, from the cases under the plan it knows of, times the remedy's cost (10-04 §1.6). No
  restriction is a hard veto (10-05 §5.3).
- **An encroachment is recorded with who saw it.** An encroachment is a building begun after the
  plan, on its reserved ground.
- **A case is brought by someone who saw it.** The case is under the plan law, heard by the
  gathering or by a forum whose `hears` names `Plan` (ADR-0025).
- **Remedies are the template's bundle:**
  - a fine in food to the store;
  - **removal**, by the crews that take buildings down (M6b slice BD), only for a building begun
    after the plan on reserved ground;
  - regularisation for a yearly payment.

  Tolerance is no case brought.
- **A use older than the plan is never an encroachment** (10-05 §1.6).

### 6. Zoning is a clause of a plan, beside nuisance rules

- **A plan's districts may carry the `PlotUse`s they allow.** A keeper builds only those there, and
  an older use stays. A settlement without a plan has no zoning.
- **Two `RuleTemplate`s are appended,** over harms the kernel models:
  - a building where fire is worked keeps a distance from others' homes (`fire.rs`'s hazard);
  - a midden keeps a distance from a well (`contagion.rs`).

  Each answers what people saw (`burned`, `sickness`), as M6b's rules do.
- **Dimensional districts wait** (coverage, storeys, setbacks) until after v1 (10-05 §5.6).

### 7. Density is measured, and height is M8d's

- **Densities are measured, never set,** by 10-08 §1.1's quantities with their denominators.
- **The dashboard's Land row** shows densities, payments by channel, plans' shares (reserved, built,
  occupied and encroached), and reckoned values by walk to the hearth. It has no tolerance until a
  run is seen.
- **Height is a household's choice in M8d,** where frame grammar v3 brings storeys: a larger
  footprint on the plot it can get, against another storey, both in hours. Plan §5.1's "how tall
  (land value, tech)" is amended: nothing reads land value, and a scarce, dear or small plot makes a
  storey worth more to the household that holds it.
- **"Planned", "organic", "grid" and "planning institution" are derived labels.** Plan §5.1 is
  amended: what the label names is a polity with a plan in force and a keeper.

### 8. The boundary and saves

- **A saves section for plans** holds each version's template, origin, orientation and realised
  layout, its clauses and answers, phases, parcels and allotments, and encroachments and cases.
- **Receipts ride with plots.** Old saves load with no plan and no receipt.
- **The observer** gets `GetPlans`, with a plans revision in the snapshot; the map's three
  reservation layers; and the land map labelled paid, reckoned or nothing recorded. A plot's page
  says why it was chosen and what was paid.

## Slices

- **CS**: where a household builds, as a choice. Candidates in hours, the receipt, the reach
  replaced; new baselines; a plot's "why here" in the observer.
- **CT**: the plan law and its reservations. `Plan`, the layout function and its first operations,
  3–8 alternatives, forecasts, the three issues, versions, keepers avoiding reserved ground,
  encroachments recorded, the reservation layers.
- **CU**: land assembly and allotment. Clauses and answers, the three channels, the keeper's office,
  phases, the four allocation rules, ground rent, build by.
- **CV**: compliance and cases. Expected sanction, sightings, cases, forums, remedies, the backlog.
- **CW**: use and nuisance rules. District uses and the two rule templates.
- **CX**: land value and density. The derived map, the Land row, a bench of the largest settlement
  M8 lives.

## Consequences

- **Easier:**
  - a settlement can outgrow 28 ha;
  - land value has a source that can be inspected;
  - plans can be proposed, half built, encroached on and repealed, with each step on record.
- **Harder:**
  - every world's homes move once, and its digests with them;
  - a household's build costs a few travel fields;
  - there is a plan section with versions;
  - the keeper is another office to elect or seize.
- **Without sales, land value stays thin until plans with fees exist.** That is the honest result
  of ADR-0007 in M8.

## Alternatives considered

- **A kind per operation** (reserve a corridor, subdivide, extend). Rejected: one law should carry
  one coherent intention, and amendments and repeal are simpler on one record (brief question 1).
- **Land sales and leases of plots in M8.** Deferred: 08-19 §5.3 puts rights and rental first, and
  ADR-0007's revisit condition is not met. Fees, rent and compensation already move goods for
  ground (question 2).
- **No *taken* clause.** Rejected: conquest, confiscation and compulsory acquisition are historical
  vocabulary (10-04 §1.2). The guard is that taking costs a grievance and never clears a home
  (question 3).
- **Keeping the geometric home rule for worlds without a plan.** Rejected: without a choice there is
  no receipt and no land value, and the reach would still cap a city (questions 4 and 5).
- **The store's keeper as the plan's keeper.** Rejected: allotment is a power worth contesting on
  its own, and a separate office lets a polity give it by lot, election or appointment (question 6).
- **A settlement founded under a plan at setup.** Rejected as plot. The M8 demo's planned capital
  must come from a plan a gathering passed in a lived world founded large by setup (M8d), or not at
  all (question 10).

## Revisit when

- Land sales, mortgages or lodging are designed (ADR-0007's revisit).
- Dimensional districts are wanted (coverage, storeys, setbacks).
- Survey error and local units of measure matter (10-04 §1.3).
- Site choice costs more than a twentieth of an Accelerated day.
