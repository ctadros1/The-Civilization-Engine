# Industry and Water Power

This guide describes the implemented part of M10a in the 2026-10-22 `main` tree. M10a is still in
progress. Fuel by heat, ironmaking, shared milling and later industrial works are not complete.
The [project plan](../../PROJECT_PLAN.md#7-milestones) and [ADR-0033](../../decisions/0033-power-fuel-and-the-works.md)
define the broader design; this page describes what is present in code.

## Where power lives

`civ-land::power::Installation` is the saved physical installation. It records a permanent ID,
river reach, wheel position, usable fall, headrace capacity, efficiency, rated output, owner and
optional driven building. The record belongs to land because it describes fixed infrastructure.
The current schema stores it in the earth section (save schema 173 introduced the section; schema
174 adds the household's mill-observation record).

The installation does not store a daily output. `Land::installation_kw` derives it from the
current day's river discharge. If `Q` is the lesser of headrace capacity and the intake's allowed
share of river flow, `H` the usable fall, and `η` the wheel efficiency, output is:

```text
power (kW) = min(9.81 × Q (m³/s) × H (m) × η, rated kW)
```

The share follows the same conservative limit used for water diversions. Water returns to the
river at the wheel's tail, so an installation does not subtract its flow from downstream reaches.
A dry reach produces no power; a low river reduces it; high flow cannot exceed the race or wheel
rating. Invalid records (non-finite values, no fall, no race capacity, or invalid efficiency) are
rejected.

## How making uses power

Recipes can set `kwh_per_unit`. During a making session, the amount produced is bounded by labor,
available materials and the energy delivered by installations owned by the worker's holder. The
kernel uses the same limit for household making and workshops. Powered recipes cannot be priced as
though the power were free: a household without an installation does not count powered output as
one of its available ways of producing that good.

The water-wheel content defines a fixed tool rated at 3 kW and 30% efficiency. The powered grain
recipe uses 0.06 kWh per kilogram and 0.02 labor hours per kilogram, so a fully supplied wheel can
grind up to 50 kg per hour before other constraints. The recipe and tool are authored content;
the kernel applies their energy requirements and the installation's current output.

## How a household chooses a mill

Each year, a household that knows the water-wheel technique compares its recent output with the
other ways it can make the same goods. It estimates the value of powered output after the wheel's
labor and material costs. It looks for a viable site downstream from an intake: a ditch carries
water to a wheel beside a lower river cell, leaves at least 0.5 m of useful fall, avoids fields,
plots and wells, and remains separated from another wheel. Its estimate uses the reach's mean
flow and eight grinding hours per day.

The household chooses a site only when its expected gain over twenty years exceeds digging the
headrace and making the wheel. It then digs the headrace as a ditch whose destination is the wheel,
not an irrigated field. The installation is recorded with that work and does not turn until the
headrace is complete. The household builds one wheel for each installation and does not build a
second mill while the first is underway. The current design has no shared mill service: households
that know the technique can each choose their own mill, and the next planned M10a part adds a toll
for grinding at another household's mill.

## Evidence and limits

- Unit checks cover the research example of 0.5 m³/s through 2 m at 50% efficiency (4.905 kW),
  low and dry rivers, race and rating caps, and invalid records.
- Powered-recipe checks compare full, reduced and absent water power and track wear on the wheel.
- Save/load checks preserve installations and mill-observation state. Power is recomputed from
  the loaded world and current river state.
- A measured seed-3 village in which every household knew the technique and ground 600 kg per
  year had all six households choose mills, with 0.5–1.0 m falls and 8–24 m headraces. This is a
  selected lived run, not a claim that six mills are typical.
- Three benchmark worlds retained the previous run's section digests except for metadata and the
  intended content change. This shows that worlds without the new technique retain their former
  behavior in those fixtures; it does not establish universal equivalence.

The observer does not set a household's site or output. It displays the kernel's saved installation
and derived power. The feature remains a first water-power step, not a complete industrial economy.
