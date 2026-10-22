# ADR-0033: Power, fuel and the works

Status: Accepted
Date: 2026-10-22
Milestone: M10
Amends: ADR-0006 §5 (a firm holds its own tools and stations, as decided and never built), ADR-0008
§3 (techniques answer needs in hours), ADR-0010 (pits below ground water)

## Context

Plan §7's M10 opens with "steam, factories". The brief (`docs/briefs/m10-power.md`) reads 07-06,
07-12, 07-05, 08-08, 08-10 and 08-03 against the code. Its findings:

- **Nothing to burn but firewood.** Hearths burn any good marked as fuel by the kilogram in
  catalogue order; firewood stands for charcoal; there is no charcoal, coal, iron or coke; copper
  is worked under the pottery skill. Pits never reach ground water.
- **Firms use their owners' tools** (`population/firm.rs`), although ADR-0006 §5 decided a firm
  holds its own tools and stations; a firm runs one line of making at a time.
- **Industrialisation is a cluster, not a switch** (07-12): dear labour against cheap fuel, a
  market wide enough, techniques that save hours where hours are dear, capital that can wait for a
  return. Power began as water at a fall (07-06 §1.3: a wheel of a few kW), then coal and the
  atmospheric engine at the pit head, where its own fuel was cheapest (07-06 §2.4, §3).
- **Every decision checks every activity**, so each new activity adds to each decision's cost;
  the engine runs about 0.7 ms a person-day at 2,000 a settlement (ADR-0032, amendment of DN).

## Decision

### 1. Fuel by heat

A good may carry a `[fuel]` table: megajoules a kilogram and the smoke it makes. A hearth's need is
heat (megajoules a day by the weather and the household), met from the fuel a household holds,
cheapest by its own hours a megajoule first; firewood is the default. Charcoal is made from wood in
a burn of elapsed days; coal and peat are dug where deposits are; coke is made from coal. Hearths
may burn coal and peat (07-06 §2.4) once a household holds them. Charcoal burning takes wood from
a patch's yearly increment first and its standing stock after; a patch cut below its floor turns
to scrub, as the habitat rules already let grazing and clearing do.

### 2. Batches in elapsed time

A recipe may name `elapsed_days`: a batch (a charcoal burn, a smelt, a coking) is started with
its inputs, runs unattended for its days and is ended by a visit. A batch is a saved record of its
station, inputs, owner and the day it ends. Hours worked stay the recipe's hours.

### 3. Iron

Iron ore at deposits, the `bloomery` (a batch), `forging` and a smithing skill; iron tools that
serve as the stone, wood and copper tools do (`serves`). Copper moves from pottery's skill to
smithing once smithing exists. Iron is goods: the save and wire carry it as any good.

### 4. The firm's stations and stages

As ADR-0006 §5 decided: a firm holds its fixed tools and machines, which wear on its books and pass
with the firm, not its owner. A firm may run several lines with goods in progress between stages.
A mill may take a toll: a share of the grain it grinds, posted as the firm's term (a law may cap it
as a privilege's term, ADR-0026), carried on an appended `Toll` channel.

### 5. Power as a derived rate at a site

An installation (a wheel on a headrace, later an engine) is a saved record: its site, intake, head
and machine. Its power in kilowatts is worked out each day from the river's discharge (ADR-0018)
or the engine's fuel, and never saved. A recipe may ask `kwh_per_unit`; a powered station makes at
the least of its hands, its inputs and its power. Two wheels on one headrace share its flow. A dry
month idles a mill and households go back to their querns.

### 6. Techniques answer needs in hours

Amending ADR-0008 §3: a technique's `[answers]` may name the hours it saves on a need (grinding,
lifting water, driving a machine). A household, firm or polity deciding whether to build weighs
those hours against the work and goods the building asks, by its own hours as every other choice
is weighed. Nothing is built where hours are cheap; nothing is found that nobody seeks.

### 7. Coal, pits and engines

A pit dug below the ground-water head floods at the rate the water table gives (ADR-0018's
aquifer); it is bailed by hand, by a wheel, or by an engine. The atmospheric engine is a technique
and a machine: it burns coal for power at a pit head, where its fuel costs least; a rotary drive is
a later technique. Deep pits do not draw down neighbouring wells in M10 (a later amendment).

### 8. The works

A works is a firm whose stations need power and crews: a machine names its hands; a missing hand
idles it. Work runs in shifts by daylight; gas light (ADR-0034) lets a works run after dark. Hired
labour is ADR-0006's wage labour at posted wages. In a household-resolved settlement a works' day
is the sum of its workers' blocks, as every behaviour is (ADR-0032 §2); a running works does not
hold its settlement Detailed.

### 9. Who owns a works, and where far techniques come from

A works may be owned by a household firm, an organization by stake (M7c slice BW) or a polity's
public work; no new partnership form is built in M10. Far techniques (coke smelting, the
atmospheric engine) are found by discovery at long E50s, as every technique is (ADR-0008), or
introduced by the observer; there is no world beyond the map to import from. Most worlds will not
industrialise, and that is a valid outcome.

### 10. Not in M10

Textiles and fibre (07-07), draught animals and horse gins, wells drawn down by deep pits, and
kilowatts and megajoules on the wire (the observer speaks hours and kilograms; a works page may
name kilowatts). Each is a §9 entry when deferred.

## Consequences

- Wood, coal and power are scarce in the land, so who industrialises follows from geography, fuel
  and the price of hours, never from a date.
- Saves gain fuel batches, installations and firm-held stations; channels, kinds and goods stay
  append-only; the save schema, wire and content API bump.
- Each new activity is measured on the bench before it is enabled; a candidate filter keeps
  decisions from scoring activities a household cannot reach.

## Alternatives considered

- **Power as a settlement-wide stat.** It cannot say why a mill stands at one fall and not another.
- **A tech tree with eras.** Forbidden: the engine authors vocabulary, not the plot.
- **Partnerships with joint capital now.** Organizations by stake already hold shares; capital that
  waits for a return follows notes and credit (ADR-0035).

## Revisit when

- The bench shows the new activities cost more than the time box allows.
- No world in the demo builds a mill where its hours are dear (the hours answer is mis-weighed).
- Textiles or draught animals are needed for a works to pay anywhere.
