# ADR-0042: Climbing, lifts, taller frames and wind

Status: Accepted (2026-10-31, by Claude under the owner's standing authority)
Date: 2026-10-31
Milestone: M11 (its deferrals; ADR-0037 §7 and its amendment)
Amends: ADR-0037 §3 (frame version 4 rises past four storeys), ADR-0031 §3 (a wind case on the
lateral mode), ADR-0030 §1 and ADR-0040 §5 (climbing enters a design's hours)

## Context

ADR-0037 §7 deferred lifts, wind and storeys past four "until anyone builds tall", and letting
storeys, the reason to build tall, is now built (ADR-0040). Research 11-14: vertical circulation is
a capacity that takes floor (§1.5), people reaching upper floors spend time climbing or waiting and
riding, and that time belongs in the hours a household reckons (§1.5's proposed rule); "tall
buildings without elevators are simply expensive to use, which makes upper floors less valuable";
lateral stiffness governs as frames get taller, wind load is `q·Cp·A` with `q = ½ρv²`, and drift
of about H/500 is serviceability, not collapse (§1.4). Priors (§2.5): storeys 3.3–4.0 m in frames,
a stair climb of 10–20 s a storey, a lift's round trip of 90–180 s at 10–20 floors, 5–10 m² of
shaft and lobby a car a floor. Frame version 3 and 4 have at most four storeys (`Level` has
Ground, Upper, Third and Fourth; their codes prefix group ids, append-only).

## Decision

### 1. Climbing is hours

A household's floor above the ground costs it climbing: each trip out and back, the climb of its
storeys at a content rate (a design prior of 15 s a storey up and 8 s down, 11-14 §2.5), times the
trips the household made (the same trips ADR-0030 §1 counts for walks). It enters a home's design
hours (ADR-0030 §1, ADR-0031 §6) for the storeys its own members live on, and a lodger's weighing of
a room (ADR-0040 §2) for the storey its floor lies on; a let is on the highest storey with floor to
spare, so upper floors let for less, as they did.

### 2. A lift is a powered fixed tool

A frame building of version 4 may hold a lift (`core:good/lift`, a fixed tool good with a rated
load, a speed and a drive's kW), opened by a technique (`passenger_lift`, requiring the steel
skeleton and an engine or the dynamo; a far technique). While its building has power that day (its
keeper's installation, engine, or a hookup on the wire, ADR-0033, ADR-0036), climbing above the
ground is replaced by a wait and a ride (a closed-form round trip, 11-14 §2.5's priors); without
power, the stair's climb applies. Its shaft takes floor on every storey it serves (a design prior
of 7 m²). Nothing else reads a lift.

### 3. Frame version 4 rises past four storeys

`Level` gains storeys five to twelve (appended codes); in version 4 a skeleton frame may have up to
twelve storeys (`MAX_STOREYS_V4`), each storey above the fourth laid out as the fourth is (one
enclosure thickness, the openings of the upper storeys). A stair stays required from three
storeys; a lift is never required. Versions 1–3 are unchanged. The wire's spaces carry the new
level codes (wire bumps).

### 4. Wind on tall frames

A frame of more than four storeys is weighed under the month's storm on its day (ADR-0012 §5) by
a wind case on ADR-0031 §3's lateral mode: the storm's gust as pressure `½ρv²` on the face (a
pressure coefficient of 1.3, a design prior) shared down the storeys; members failing under it give
way as any group does, and a drift past H/500 (a design prior, 11-14 §2.4) shows as a symptom.
Wind bracing (rigid joints) is the skeleton's own stiffness here; braced variants wait.

### 5. The boundary

The lift good, its technique and recipe, the climbing priors (content `[climbing]` on the people
profile) are content (content API bumps); `Level`'s new codes ride the wire (wire bumps); no new
saved section (a lift is a held tool; storeys are in the spec).

## Consequences

- Upper floors cost hours: lodgers pay less for them, and a household weighs a second storey
  against its climb, as people did.
- A lift makes a tall frame's upper floors near as cheap as its ground floor where power is to be
  had; without power, height costs climbing.
- Whether anyone builds above four storeys follows from rent, ground and climbing; most worlds
  will not.

## Alternatives considered

- **A height cap per era.** Forbidden.
- **A lift as a building flag.** It needs power, takes floor and has a capacity; a flag would
  erase all three (11-14 §1.5).
- **A new tower grammar.** Rejected in ADR-0037 for duplicating version 3's assembly; extending
  version 4's storeys keeps one grammar.

## Revisit when

- Frames above eight storeys appear in lived worlds: lift groups, express zones, braced frames.
- Glass is made: curtain walls change a face's wind load and weight.

## Amendment (2026-10-31): as built

Decided by Claude under the owner's standing authority, by the time box (plan §9):

- **§1, reckoned, not lived.** The climb enters a home's design hours (the shapes weighed with
  their ground, ADR-0031 §6) for the storeys its members would sleep on, filled from the ground up
  (`floor_base_m2` first, then `floor_m2_per_sleeper` each), and a lodger's weighing of a room for
  the storey the room lies on, the highest storey of the home with floor to spare, lets filled from
  the top down. People's days do not spend it: nobody's day is longer for living upstairs. A
  design's rent credit (ADR-0040 §5) is not discounted by the storey the spare floor lies on; the
  lodger's weighing discounts it, and asking rents follow.
- **§2, a light-traffic wait, and the quicker of the two.** A household's few calls against a
  car's load seldom queue, so a trip out and back by lift is twice a wait for the car to come half
  the building and a ride to the ground, each with a stop (a car calling at every one of ten
  storeys of 3.5 m runs its round in about 130 s, inside 11-14 §2.5's 90–180 s); a person takes
  the lift only where that is quicker than the stair, so a lift never makes a floor dearer. Power
  that day is the keeper's installations' and fuel engines' shaft, for four hours on the fuel it
  holds (a design prior), at least the drive's kW, or a hookup of the keeper's on a wire that was
  served. The lift burns no fuel, draws nothing in the wire's dispatch and pays no meter; its rated
  load is recorded and not read (no capacity, no queue). A household's whole lifts stand in its
  finished frame buildings of version 4 and more than one storey, one each, the tallest first; the
  shaft's 7 m² comes off every storey's sleeping floor. Nobody wants a lift (`per_worker` 0, as a
  settling tank): only a household given one, or one that knows how and makes it, holds one. A
  home's design does not weigh a lift (homes are designed at version 3).
- **§3, a skeleton program may allow twelve storeys.** A frame program that offers a skeleton may
  give storeys up to twelve; its builders design a shape above four storeys as a skeleton where
  they know it, however dear (no timber frame stands so tall); a version 4 expansion may have 256
  groups. The core's skeleton programs (workshop, hall) keep one storey and its homes have no
  skeleton, so no lived world can build above four storeys yet. The wire carries no spaces: a
  group's id carries its level's code in its top byte, and wire 1.111 says the new codes may come.
- **§4, posts, square on the broad face, the day it blows.** The wind case checks posts only
  (beams and joints are the skeleton's rigid joints, unchecked), takes the storm square on the
  building's broadest face whatever its direction, leaves the roof and gables out of the face, and
  reckons the sway from sound sections. A sway past H/500 shows on the posts of the storey swaying
  most for the storm's day; the next calm day it is gone. The designer does not size posts for
  wind.
