# BD ignition sensitivity, 14 October 2026

A five-year seed-2 valley on a 512-cell map, with 40 founders and the same world identity in each arm. The behavior is from `6ca4698`; this slice adds the runner and reporting correction. Save schema 99, content API 86, wire 1.71. Base content fingerprint: `c8510aaa4cc1b6b4067d3f5fe50d9524e0be4ccfd760c0d5206df16c2ad55f9b`.

From `kernel/`:

```sh
cargo run --release -p civ-sim --example fire_sweep -- --years 5 --seed 2 --people 40
```

Nominal settings 1, 3, 10, 30 and 100 multiply all three core activity hazards relative to the 10/K household-year reference (research 12-03 §2.5, §5.7). They do not prescribe annual incidents, losses or survivors. Actual frequency follows fire use and moisture. Exposure counts living households daily. There are no observer inputs or modified-content snapshots. The runner permits a requested 10–200-person band only inside the harness, as bench_village does.

| Nominal/K | HH-years | Escapes | Observed/K | Put out | Roof fires | Spread | Deaths | Water, L | Work, h | Noticed / unnoted roofs | Living |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 61.071 | 0 | 0.000 | 0 | 0 | 0 | 0 | 0.000 | 0.000 | 0 / 0 | 45 |
| 3 | 61.071 | 1 | 16.374 | 0 | 1 | 0 | 0 | 0.000 | 0.000 | 0 / 1 | 47 |
| 10 | 61.071 | 1 | 16.374 | 0 | 1 | 0 | 0 | 0.000 | 0.000 | 0 / 1 | 47 |
| 30 | 61.071 | 2 | 32.749 | 1 | 1 | 0 | 0 | 0.000 | 0.000 | 0 / 1 | 47 |
| 100 | 61.071 | 6 | 98.246 | 4 | 3 | 1 | 1 | 45.000 | 0.167 | 2 / 1 | 44 |

[Raw CSV](fire-ignition-sweep.csv) also reports goods lost, stripping, salvage, completed demolitions and conditional first-notice time. No arm recorded stripping, salvage or completed demolition. At 100, goods loss was 1,228.571 kg and ten minutes of response work applied 45 L. Two of its three roof fires were noticed, with a conditional mean first notice of 0.50 minutes; the third has no recorded notice. Blank means are not zero-delay responses.

Each arm supplies only about 61 household-years. Zero events at 1 and single-event rates at 3 and 10 leave substantial uncertainty; 16.374/K at those settings is one event, not a calibrated annual frequency. This is a small sensitivity check, not historical validation or complete response calibration. No core hazard is tuned from it. The core reference remains an order-of-magnitude prior.

The earlier uncommitted preparation produced the same aggregate outcomes before the final physical-scene correction; those results remain in the development journal. This published CSV comes from the final BD behavior. World generation is reproducible for a seed on one build; simulation replay across builds is not promised.
