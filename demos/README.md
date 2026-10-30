# TCE Demo Gallery

This is the guide to the project's demonstrations across milestones. The watchable videos have
opening titles and on-screen explanations; their source captures are kept for production history.
A screenshot or demo run shows one build and one world, not an outcome every world will produce.

## Newest completed milestone: M9, many settlements

M9 keeps each person as an individual record while settlements away from the observer's focus are
lived by household. The full lived report is a ten-year run: ten founding groups became eleven
settlements, the population grew from 2,000 to 2,576, and the saved world reloaded with all people
present. Its measured rate was 0.039 simulated years per real minute; the 100,000-person gate was
not met. See the [plan](../PROJECT_PLAN.md#7-milestones) for the limit and run conditions.

The titled 1080p video is a separate, 29-second observer capture prepared to show the map,
focus-based resolution labels and climate state. It is not the ten-year report run. The video
records the observer after a one-year, 500-person lived run, with ten settlements. Focus-based
resolution is a session setting rather than a save field, so the capture turns it on after loading
and advances one day to show its effect. The edit trims the initial loading and run-ahead dialog;
it does not present those screens as simulation behavior.

<p align="center"><a href="m9/m9-overview.mp4"><img src="m9/m9-overview-poster.png" alt="M9 live demo: many settlements and household resolution" width="900"></a></p>

- [Titled M9 demo video](m9/m9-overview.mp4)
- [Settlement-resolution screenshot](m9/m9-settlements.png)
- [Climate screenshot](m9/m9-climate.png)
- [One-year video capture run report](m9/m9-capture-run.txt)
- [Ten-year M9 run report](m9/m9-demo.txt)
- [M9 capture runner](../web/e2e/m9-demo.spec.ts)
- [Video production project and refresh steps](m6a/production/README.md)
- [Aggregate resolution technical guide](../docs/development-journal/simulation-model.md#multiple-settlements-and-contact)

## Current development: M10, the industrial demo (run report only)

M10's demo runner is `kernel/crates/civ-sim/examples/m10_world.rs`. Its first full run was cut by
the homelab job's six-hour limit after eleven of fifteen years, so it has no final save and load:
three taught settlements set water mills, wrote records and one issued notes; the three controls
took nothing up. There is no M10 video yet.

- [Eleven-year M10 run report (cut short)](m10/m10-demo.txt)

M11's demo runner is `kernel/crates/civ-sim/examples/m11_world.rs`: six settlements of 120, three
taught M11's vocabulary by the observer and three as controls. Its third eight-year run found copper
by prospecting at Sedgehollow and came to want a dynamo in two settlements, but built no instrument,
dynamo or line in eight years; the controls lived as before, and the world saved and loaded with
all 909 people. See the plan's M11d entries for what each run showed.

- [Eight-year M11 run report](m11/m11-demo-8y.txt)
- [Industry and water-power guide](../docs/development-journal/industry-and-power.md)

## Earlier titled observer video: M6a, water and sickness

The M6a video follows cholera through household wells and separates the modeled water routes from
residents' beliefs and the polity's response. The gallery keeps its screenshots, source recording
and run log together.

<p align="center"><a href="m6a/m6a-overview.mp4"><img src="m6a/m6a-overview-poster.png" alt="M6a live demo: a close view of the village, with chapter captions" width="900"></a></p>

- [Incidents panel](m6a/m6a-incidents.png)
- [Outbreak evidence and public response](m6a/m6a-outbreak.png)
- [Water lens close-up](m6a/m6a-lens.png)
- [1080p overview with title and chapter captions](m6a/m6a-overview.mp4)
- [Recorded demo narration and observed outcome](m6a/m6a-demo.txt)
- [M6a demo runner](../web/e2e/m6a-demo.spec.ts)
- [Video production project and refresh steps](m6a/production/README.md)

## Demos by milestone

| Milestone | Demonstration | What is available here |
| --- | --- | --- |
| M0 · Foundations | Generate a seeded valley and inspect its terrain and river network | [Watch the titled 1080p overview](m0/m0-overview.mp4)<br><a href="m0/m0-overview.mp4"><img src="m0/m0-overview-poster.png" width="280" alt="M0 title card: Make a world"></a><br>[Screenshots](../assets/m0/) |
| M0 · Save recovery | After an unclean shutdown, the observer offers its last autosave and restores the world | [Watch the titled 1080p explanation](m0/m0-recovery.mp4)<br><a href="m0/m0-recovery.mp4"><img src="m0/m0-recovery-poster.png" width="280" alt="M0 title card: Recover an interrupted world"></a> |
| M1 · A band settles | Follow a founding band through village growth, then see the save and reload flow | [Watch the titled 1080p overview](m1/m1-overview.mp4)<br><a href="m1/m1-overview.mp4"><img src="m1/m1-overview-poster.png" width="280" alt="M1 title card: A band settles"></a><br>[Screenshots](../assets/m1/) · [Demo runner](../web/e2e/demo.spec.ts) |
| M2 · Unreal groundwork | Portable kernel and C interface; Unreal client remains future work | [C interface guide](../docs/development-journal/persistence-and-interfaces.md#c-interface) |
| M3a · Village economy | Compare household fields and village fields in one valley | [Watch the titled 1080p overview](m3a/m3a-overview.mp4)<br><a href="m3a/m3a-overview.mp4"><img src="m3a/m3a-overview-poster.png" width="280" alt="M3a chapter preview: Household fields"></a><br>[Demo captures](m3a/recording/), [demo runner](../web/e2e/m3a-demo.spec.ts) |
| M3b · Knowledge and buildings | Introduce a craft, inspect its history and see village buildings | [Titled 1080p overview](m3b/m3b-overview.mp4)<br><a href="m3b/m3b-overview.mp4"><img src="m3b/m3b-overview-poster.png" width="280" alt="M3b preview: framing enters the valley"></a><br>[Screenshots and source capture](m3b/recording/), [demo runner](../web/e2e/m3b-demo.spec.ts) |
| M3c · Seasons and time | Compare seasonal records, markets and accelerated simulation | [Titled 1080p overview](m3c/m3c-overview.mp4)<br><a href="m3c/m3c-overview.mp4"><img src="m3c/m3c-overview-poster.png" width="280" alt="M3c preview: read the weather years"></a><br>[Year-at-Max capture](m3c/recording/m3c-a-year-on.png), [demo runner](../web/e2e/m3c-demo.spec.ts) |
| M4 · Councils, law and crime | Government, public decisions and the path from incident to case | [M4a government overview](m4a/m4a-overview.mp4)<br><a href="m4a/m4a-overview.mp4"><img src="m4a/m4a-overview-poster.png" width="280" alt="M4a preview: a village gathering takes shape"></a><br>[M4a runner](../web/e2e/m4a-demo.spec.ts)<br>[M4b incidents overview](m4b/m4b-overview.mp4)<br><a href="m4b/m4b-overview.mp4"><img src="m4b/m4b-overview-poster.png" width="280" alt="M4b preview: an incident follows its path to the gathering"></a><br>[M4b runner](../web/e2e/m4b-demo.spec.ts) |
| M5a · Neighboring settlements | Founding groups, migration and movement across settlements | [Titled 1080p overview](m5a/m5a-overview.mp4)<br><a href="m5a/m5a-overview.mp4"><img src="m5a/m5a-overview-poster.png" width="280" alt="M5a preview: the year-ten migration snapshot"></a><br>[Demo captures](m5a/recording/), [demo runner](../web/e2e/m5a-demo.spec.ts) |
| M5b · Trade and diffusion | The original connected-village demo no longer qualifies after corrected river routing separated its villages; a replacement lived demo remains open | [Development record](../docs/development-journal/development-history.md#2026-10-10--m5b-slice-as-the-m5b-demo) |
| M5c · Relations and public works | Agreements and crossings are covered by lived runs and test worlds; no standalone observer recording is committed | [Project plan evidence](../PROJECT_PLAN.md#9-decisions-log) |
| M6a · Water and sickness | Cholera, water sources, resident knowledge and the Water lens | [Watch the titled 1080p overview](m6a/m6a-overview.mp4) · [screenshots and run log](m6a/) · [demo runner](../web/e2e/m6a-demo.spec.ts) |
| M6b · Fire, masonry and floods | Fire response, masonry, floods, banks, ditches, storms and building rules | [Screenshots and run report](m6b/) · [M6b status and evidence](../PROJECT_PLAN.md#7-milestones) |
| M6c · War and conquest | Two villages of 100 raid each other; one gathering raises a palisade, makes a law against raiding and sends bands by law, until a band holds the other village (screenshots of the siege page, the map and the Government panel, with the run log; two priors nudged, both logged) | [Screenshots and run log](m6c/) · [demo runner](../web/e2e/m6c-demo.spec.ts) |
| M7a · Councils and elections | A village changes its custom and elects a council in the selected run | [Screenshots and run record](m7a/) · [demo runner](../web/e2e/m7a-demo.spec.ts) |
| M7b · Courts and precedent | Court, precedent and review mechanics; no titled observer video is committed | [Run record](m7b/) · [demo runner](../web/e2e/m7b-demo.spec.ts) |
| M7c · Organizations and trade policy | Duties and embargoes passed in the recorded run; no organization emerged at that scale | [Run record](m7c/) |
| M7d · Credit before coin | Four-year run, three villages; loans emerge, no copper or coin in the featured seed | [Run report](m7d/m7d-demo.txt) · [demo runner](../web/e2e/m7d-demo.spec.ts); observer recording not captured yet |
| M8 · Growing city and structural response | Completed: transport, planning, structural behavior and public buildings; engineering/run evidence, no standalone observer video | [Current status and evidence](../PROJECT_PLAN.md#7-milestones) |
| M9 · Aggregate LOD and scale | Ten-settlement lived run and titled observer video; the 100,000-person speed target was missed and amended | [M9 video, report and runner](m9/) · [run details](../PROJECT_PLAN.md#9-decisions-log) |
| M10a · Power and the works | In progress: water-power installations, powered recipes and household-built mills; no milestone video | [Technical guide](../docs/development-journal/industry-and-power.md) · [current plan](../PROJECT_PLAN.md#7-milestones) |
| M12 · Model deliberator and viewers | Partly implemented: a stand-in policy (not a model) chooses at the polity review and a recorded replay relives its choices; a driver and a spectator page on one host. Close-up fidelity awaits the Unreal client | [Run report and screenshots](#m12-2026-10-30) · [runner](../kernel/crates/civ-sim/examples/m12_world.rs) · [capture spec](../web/e2e/m12-demo.spec.ts) |

Some milestone demo scripts intentionally explore emergent outcomes and can take a long time. They
run only when requested with `TCE_DEMO=1`; they may try more than one world and report what
happened instead of forcing a scripted result. For milestone scope and known limitations, use the
[project plan](../PROJECT_PLAN.md). For the complete chronological record, see the
[development journal](../docs/development-journal/README.md).

## Refreshing the latest demo

When the current README status or feature showcase is updated, update this gallery and the README's
featured images together. Use the newest completed demo that has a real observer run. Save fresh
captures under `demos/<milestone>/`, keep prior milestone media and links in this gallery, and label
concept art separately from screenshots. Do not describe an unfinished milestone's design as a
demo.

From `web/`, prepare and record the M9 observer demo with:

```sh
TCE_DEMO=1 TCE_DEMO_VIDEO=1 TCE_SHOTS_DIR=../demos/m9 npm run test:e2e -- m9-demo.spec.ts
```

The runner creates a one-year, ten-settlement lived save, then records the loaded observer at
1920×1080 and writes screenshots into `demos/m9/`; the capture run report is saved as
`m9-capture-run.txt`. For the titled edition, copy the raw source into the Remotion project's
ignored `public/` directory and render from `demos/m6a/production/`:

```sh
cp ../../m9/m9-demo.webm public/m9-source.webm
npm run render:m9
```

Keep the ten-year lived report `demos/m9/m9-demo.txt` distinct from the short video capture's
`m9-capture-run.txt`; do not overwrite the longer report when refreshing the video.

Keep a newer demo feature here and in the top-level README only after a lived observer recording
has been captured. Engineering fixtures, benchmark reports and milestone plans are valuable
evidence, but do not count as observer demos.


## M8 (2026-10-20)

The M8 demo is `civ-sim --example m8_city <saves> 1 500 1080`, recorded in `demos/m8/m8-demo.txt`:
two settlements of 500 on the river valley, the port and an inland one, live three years with
framing introduced by the observer on the first day and the ground shaken once. No household built
a frame; the polities raised two granaries and a hall; a plan of streets was rejected at first and
passed later at the port; the shaking felled nothing. `m8_city <save>` reads the run's save again.
The scale gate's figure (ADR-0028, amendment) is measured with `bench_village`.

## M9 (2026-10-21)

The M9 demo is `civ-sim --example m9_world <saves> 10 200 10 2048`, recorded in
`demos/m9/m9-demo.txt`: ten groups of 200 on a map of 2048 cells, one person followed in full
detail and every other settlement lived by household, the climate forced 3 °C warmer and a fifth
drier over twenty years, lived a decade at the fastest speed. The people founded an eleventh
settlement in the third year; 2,000 became 2,576; three people of household-resolved settlements
opened at the end each have their name, household, parents, knowledge and the block of the day
they are in, and the save of 28 MB loads with everyone there. It ran at 0.039 simulated years a
real minute. The gate's figure at 100,000 (ADR-0032, amendment of slice DN) is measured with
`bench_village`.

## M12 (2026-10-30)

The M12 demo is `civ-sim --example m12_world <saves> 2 3 2 100 1024`, recorded in
[`demos/m12/m12-deliberator.txt`](m12/m12-deliberator.txt): two founding groups of 100 on a map of
1024 cells, saved on their founding day, then that save lived two years three times. First by rules
alone; then with a **stand-in** for a model, a fixed policy written for the demo (it proposes the
listed move with the most expected support when that support is at least one half, and otherwise
does not answer, so rules choose), saved at the end; then from the founding save again in `Recorded`
mode, answered from the second run's save. No language model was run. Rules alone proposed 64 laws,
of which 39 passed (in force or since ended). With the stand-in, 64 reviews offered moves: it
proposed at 61 and left 3 to rules, which proposed at 2; 64 laws were proposed, not all the same
ones, and 50 passed. The recorded replay read the 63 saved answers, asked no provider, and ended
with all 65 save sections byte-equal to the stand-in run's. In every run the people fell from 210 to
about 150 in the second year. The three runs took 48 minutes here in the `simcheck` profile. What
the gatherings did with each proposal was theirs; the report lists every law in the government
panel's words, including "chose by the model" or "chose by rule (the model did not answer)".

The two screenshots are one host with two pages on a small world (`TCE_DEMO=1
TCE_SHOTS_DIR=../demos/m12 npm run test:e2e -- m12-demo.spec.ts` from `web/`): the first page drives
and has the hand-over list; the second watches, says who drives, and has every command control
disabled.

- [Run report: rules, the stand-in and the recorded replay](m12/m12-deliberator.txt)
- [Driver page](m12/m12-driver.png)
- [Spectator page](m12/m12-spectator.png)

Close-up fidelity (MetaHuman-grade notables) is Unreal work and waits, with M2's client, for the
owner's Windows PC.
