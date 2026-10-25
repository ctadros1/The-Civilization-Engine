# Industry and Power

This guide describes implemented M10a systems in the integrated `main` tree as of 2026-10-24. M10a, power and the works, is closed. M10b adds rail and gas light; M10c services and letters is underway. The [project plan](../../PROJECT_PLAN.md#7-milestones) and [ADR-0033](../../decisions/0033-power-fuel-and-the-works.md) govern scope and evidence. The design follows the research distinction between fuel heat, mechanical power and energy services ([research 07-06](../../research/07-technology/06-energy-power.md)); code parameters remain authored model choices, not historical measurements.

## Water power and powered work

`civ-land::power::Installation` stores fixed river infrastructure: reach, wheel position, usable fall, headrace capacity, efficiency, rated output, owner and optional driven building. Daily output is derived from river discharge, the available share of flow, fall and efficiency, capped by race and wheel ratings. It is not separately stored as daily state. Water returns to the river below the wheel.

Recipes can require energy. Production is bounded by labor, materials and energy available to the worker's holder. The authored wheel and recipe parameters determine potential output; the kernel applies them to current flow and ownership. This is the physical and economic path from a river to powered production.

Households that know the water-wheel technique can weigh a site and investment against expected future work savings. A viable choice requires a headrace and useful fall without conflicting with nearby land uses. Once it chooses, the household digs the race, builds the wheel and can use it when construction completes. A household may also carry grain to a neighboring working mill and pay a toll; its choice compares this service with the cost of its own installation.

## Fuel, iron and works

Fuel is accounted for by heat delivered, so different fuels can serve a recipe according to their authored properties. Some recipes run in batches without a person continuously attending them. Iron production begins with bog ore, a bloomery and forging under the relevant skills and techniques; people do not start with those capabilities.

Workshops can have multiple production lines and stations that belong to the works, with goods in progress. A machine can require a crew to be present together. A works with a machine can call its crew at sunrise and hire for a shared shift; household-resolved work handles the crew in aggregate. Coal seams, pits, flooding and bailing provide a route into coal extraction. An atmospheric engine can pump a pit using coal heat, and water wheels can power machinery. Coke and a wheel-driven blast furnace extend the chain to pig iron and bar.

These features make industrial production possible through resources, techniques, recipes and household decisions. They do not cause every world to industrialize, and the project does not claim calibrated industrial history.

## Evidence and limits

The project plan records each slice's save version, content API, observer boundary and selected evidence. Unit and integration checks establish local rules and persistence behavior; selected lived worlds show that particular pathways can occur. They do not establish typical outcomes across seeds or historical validity. The M9 observer video remains the newest committed milestone video; M10's full demo is planned for M10d.

The observer displays kernel-authored installations, output and workshop state. It forwards commands and does not choose a site, fuel, machine or production outcome. See [persistence and interfaces](persistence-and-interfaces.md), [development and evidence practice](development-and-evidence.md), and the [M10 milestone entry](../../PROJECT_PLAN.md#7-milestones).
