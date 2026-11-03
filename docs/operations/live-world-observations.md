# Live world observations

The private runner dashboard can render the actual world being advanced by a headless job.
This is a sampled top-down operations view, separate from the interactive web observer. It
never creates another world from a seed, loads an unrelated save, or sends world commands.

## Enabled paths

`homelab.yml` enables `TCE_OBSERVATION_DIR=$RUNNER_TEMP/tce-observations` only in the long
simulation step and milestone demo step. `nightly.yml` enables it for self-hosted smoke,
consistency/notables and sanity-dashboard simulation steps. `civ-host run` supports the same
opt-in environment. `m10_world` publishes from its daily loop. Other/new examples must add an
`Observation::from_env(&sim)` publisher and call `poll(&sim)` at advance boundaries; merely
setting an environment variable does not instrument an older revision.

Benchmarks, ordinary test suites and chains do not opt in. `BENCH_GATE` overrides any inherited
observation setting. The exporter is not called by `Sim::advance_minutes` or benchmark examples.
LongRun retains its original month advance when disabled; observed runs expose daily advance
boundaries for sampling, while economy sampling remains at the existing monthly boundaries.

The workflow YAML and the code under test must both include the exporter integration. For
routine dispatch, use main's workflow with `ref=<branch-or-full-SHA>` containing it. A scratch
workflow may also opt in while preserving `tce-ci`, current routing and the six-hour node4 limit.
Older running or queued processes keep their original code and are not retrofitted or cancelled.

## Watch a short real world

Dispatch `homelab.yml`, `suite=world-view`, `runner=tce-ci` (or an explicit node), from a revision
containing this workflow and exporter. It builds `observe_world` and advances an actual
80-person, 256-cell world for three real minutes. The job has a 45-minute build/runtime budget;
it uses normal scheduling, respects node4 closure and does not cancel other suites. It prints
the successful frame count and cumulative exporter wall time when finished.

For local use, from `kernel/`:

```sh
TCE_OBSERVATION_DIR=/tmp/tce-worlds cargo run --release -p civ-host -- run --seed 2 --years 1
```

The dashboard expects the runner-temp path, not the local example path. Select the running
node and a world in the pane. Whole map / Settlements, zoom and drag only change the camera.
After a world exits its file is removed; crashed/stale publishers are hidden, not called live.

## Observation format and bounds

`civ-sim::observation` produces version 1 JSON, one file per world/pid/instance. A temporary file
is renamed atomically within the directory. Fields include world and stream identity, actual
name/seed/preset/mode, Unix sample time, publisher pid, day/minute, extent in metres, entity
coordinates/ids/names and total entity counts. Numeric ids and seeds use strings to preserve
64-bit precision in JavaScript. People use the existing observer's day-block/position rule.

Terrain is a sampled raster of at most 128 by 128 actual map cells: relative elevation 0..255
and the engine's water classes 0 land / 1 river / 2 lake / 3 ocean. Small features can disappear
at this overview resolution. Point samples cap at 100 settlements, 4,000 buildings and 8,000
people; totals disclose truncation. Every frame is below 2 MB or publishing disables itself
with an output warning. No ids/RNG draws or world mutations occur while building a frame.

Publish attempts occur at most once per five wall seconds, at caller advance boundaries. A
single slow advance may delay the next frame; the dashboard hides frames older than 120s.
The initial 30-day smoke phase also exposes daily boundaries when observation is enabled.
I/O failure disables only that publisher and never fails the simulation.

The collector requires an active Actions worker, a frame modified after that worker began,
and a live publisher in the runner service tree. It reads at most 12 frames / 8 MB per poll;
the largest multi-world runs may therefore show a subset. The dashboard validates bounded
finite coordinates, terrain dimensions/classes, and current timestamps (including future
rejection). The map is entirely view only. It has no WebSocket or inbound guest listener.

## Validation

Observation tests compare save section bytes before/after publication and the future state of
the same saved world advanced monthly versus daily with observations. They also check actual
positions/terrain, atomic files, rate limiting, cleanup, disabled operation and write failure.
The short Actions world verifies the real process → file → guest collector → management API →
canvas path separately from simulation unit checks. Deployment evidence lives in the private
homelab record; do not equate unit tests with live rendering.

Validated on 2026-11-03: the production dashboard rendered the real Actions world in run
[41679635673](https://github.com/ctadros1/The-Civilization-Engine/actions/runs/41679635673),
with world time advancing and changing canvas pixels. Renderer checks reported no browser
errors or write requests; node3's pre-existing worker PID remained unchanged. Node4 was already
closed and offline; its protected collector update is deferred until reconnect without reopening it.
Kernel regression: 1,390 tests passed plus the added same-world stream isolation test; 3 ignored.
Commons: 31 tests; web: 213 unit tests and 30 browser tests (15 existing skips); all 10 standard smoke
worlds passed. Observation regression checks 4 cases including unchanged future/save state.

The repository's existing schema check exposed mixed trailing whitespace in committed generated
files. The generator now canonicalizes trailing spaces, and its outputs were regenerated; the
exact freshness check passes and `git diff -w` confirms no schema/generated meaning changed.
Python 3 is needed for this formatting step as well as the existing operations tooling.
