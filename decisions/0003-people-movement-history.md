# ADR-0003: People, movement and history in the kernel, saves and boundary

Status: Accepted
Date: 2026-10-03
Milestone: M1

## Context

M1 adds people: 20–50 at first, 10–50k by M8 (plan §4.3). They need needs, daily life, trips,
families, births and deaths, and a history the observer can question: "why did she do that", "who
were his parents", "what happened in year 3" (plan §2, §4.1). Three things become expensive to
reverse once worlds are saved and Unreal reads the stream in M2:

1. how a person's state is kept and saved;
2. how movement is described on the boundary;
3. what history the kernel keeps, and for how long.

Reports read: 01-03 (multi-rate time), 01-07 (state streaming), 01-08 (routing), 01-09 (decision
architectures), 02-01/02/03/05/07 (reference simulations), 15-01 to 15-04 (observer UX), and the
people, demography and kinship reports (04-01/02/03/04/07/08/12, 05-01/02, 06-01/07).

Their main points:

- **Event scheduling only works if stale work is harmless.** Every scheduled event must carry the
  version of the activity it belongs to; interrupting an activity bumps the version, and old
  events then do nothing (01-09 §4.1, 01-03 §4.1).
- **Needs are lazy.** A need is stored as a value, a rate and the time it was last updated, and
  schedules its own threshold crossing; nothing is decremented every minute (01-09 §3, 02-01 §4.6).
- **Trip start and end are not enough.** Trips get interrupted and replanned; a renderer that only
  knows "left at t0, arrives at t1" cuts corners and teleports. Movement must be described as a
  path with timings and revisions (01-07 §1.5, §4.4; plan §8 open question 12).
- **Explanations must be recorded, not reconstructed.** The evaluator's own numbers are copied when
  a decision is made; explaining an old decision with today's values is wrong (15-03 §1.2, §4.1).
- **History needs retention limits.** Logging every activity for 50 people over 10 years is about
  180 MB (02-03 §6). Routine records live in bounded rings; life events live forever.

## Decision

### 1. People in the kernel

1. **Plain tables with generational handles** (`civ_core::GenTable`), one row per living person
   and per household, owned by `civ-agents`. Every person and household also has a
   **`PermanentId`** that is never reused. History, the boundary and saves refer to people by
   permanent id; handles are an in-memory detail.
2. **Activities are versioned records.** A person has one current activity
   `{kind, target, started, ends, version}`. Every scheduled event carries `(handle, version)`.
   Interrupting or replacing an activity increments the version; an event whose handle is dead or
   whose version is stale is ignored. Durations are at least one minute.
3. **Needs and body state are lazy**: `(value, rate, updated_at)`, advanced in closed form when
   read. A need that will cross a threshold schedules that crossing as an event.
4. **Decisions happen only at events**: an activity ends, a trip ends, a threshold is crossed, a
   commitment falls due. Choice is utility-based over an authored activity catalogue, with
   softmax sampling (plan §4.1).
5. **Randomness is keyed, not streamed.** Each draw comes from
   `Rng64::from_key([world seed, purpose, permanent id, counter])`, and the counters are saved.
   Determinism is still not a goal (plan §1 rule 6); keying makes a save and load exact and makes
   tests reproducible.
6. **Life hazards** (death, conception) are daily draws of `1 − e^(−h·Δt)` from keyed streams at
   day boundaries, so splitting an advance into pieces changes nothing.

### 2. Movement on the boundary

1. A **trip** is `{trip id, person, route, departs, cumulative minutes per route vertex, revision}`.
   The route is a polyline in world metres, simplified to about 4 m. The arrival time is the last
   cumulative entry. Positions at time `t` are interpolated along the route by the observer.
2. **Changes are revisions**: an interrupted or replanned trip keeps its id and gets a new revision
   whose route starts at the person's position at that moment. A trip that ends early carries an
   end reason.
3. **Delivery in M1 (tens of people):** the replaceable snapshot carries one compact entry per
   living person (permanent id, position at the snapshot's time, activity, trip id and revision).
   Trip geometry is fetched by query when an observer sees an id or revision it does not have.
   **From M2**, trip lifecycle moves to the ordered delta stream (ADR-0001 frame kind `Delta`) so
   that per-snapshot size does not grow with population; the trip record itself does not change.

### 3. History

| Record | Holds | Kept |
|---|---|---|
| Person record | permanent id, name, sex, birth, death and cause, parents, origin | forever |
| Chronicle event | sequence number, time, kind, place, participants by permanent id, facts, cause links | forever (an error past 100k) |
| Decision receipt | needs at the time, chosen option and runner-up with each consideration's contribution, two more totals, exclusions, probability, draw, outcome | ring of 64 per person; life and building decisions forever |
| Memory | up to 16 salient episodes per person (time, kind, valence, subject, chronicle link) | while the person lives |

- Chronicle text is rendered by the kernel from content templates into **spans** (text or an
  entity link). The observer never builds explanations of its own.
- A chronicle reference to a person always resolves, alive or dead, through the person record.

### 4. Saves

- New `commons-persist` sections (ADR-0002): `people`, `houses` (households), `history`
  (person records, chronicle), `receipts`, `events` (the scheduler's pending events), plus the
  land and settlement sections listed in ADR-0004. Every section is versioned; derived state
  (kin indexes, road polylines, nav costs, participant indexes) is rebuilt on load.
- **M0 saves still load**: their missing sections mean "no people yet", which is a valid world.

## Consequences

- Most people cost nothing in most minutes; 50k people stay possible without changing the model.
- Stale-event bugs are structurally prevented, at the cost of a version check in every handler.
- Saves grow with history, but boundedly: about 1 MB per 50 people per decade.
- The observer can always answer "why" from data, and must say "not recorded" when it cannot.
- Unreal (M2) consumes trips exactly as the web observer does; only the delivery stream changes.
- Forbidden from now on: per-minute polling of every person; positions sampled per frame as the
  movement contract; explanations computed in TypeScript; chronicle entries made from text.

## Alternatives considered

- **An ECS framework.** Rejected in plan §3.2; plain tables with generational ids are proven in
  Genesis and Prometheus.
- **Sampled positions in every snapshot.** Simplest, but it cuts corners at 10× (one snapshot spans
  96 simulated seconds) and grows linearly with population.
- **The ordered delta stream for trips now.** Right for 50k people, but it needs gap detection and
  resynchronisation in the web client before M1 has a population large enough to need it.
- **Logging every activity.** Unbounded; 180 MB per decade at M1 scale.
- **Streamed (unkeyed) randomness.** Cheaper, but a save and load would change every future draw.

## Revisit when

- The snapshot's per-person entries exceed about 64 KB (roughly 1,500 people): move them to deltas.
- A decision needs more than eight considerations or a receipt grows past 512 bytes.
- The chronicle cap is reached in ordinary play.
