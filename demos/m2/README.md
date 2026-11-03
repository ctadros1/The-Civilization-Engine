# M2: First light in Unreal

[Watch the titled demo](m2-first-light.mp4) · [Source recording](m2-first-light-raw.mp4) ·
[Night screenshot](first-light.png) · [Construction screenshot](construction.png)

The standalone Windows client creates seed 12, a 512×512 world at 8 m per cell
(16.78 km²), with 32 founders. They live for 90 days through the normal kernel, then
the observer follows Senna at 1× on May 30 at night. Completed homes use exact grammar
parts and the first Blender kit; hearth light requires a built hearth and household fuel.
The inspector, camera and geometry all read the same in-process kernel. No buildings,
stocks, events or completed construction were inserted for the capture.

The approximately ten-second, 1080p video uses **224 actual native JPEG captures**, targeting
30 captures per second. File timestamps retain their observed wall-clock pacing; the MP4
is encoded at 30 fps. Titles are the only visual edit. This recording path has readback and
encoding overhead, so its frame rate is separate from the renderer's 1440p60 acceptance run.
The unedited 1440p night screenshot comes from that acceptance run. The daytime construction
capture is a separate naturally advanced world, with the camera at an actual building plot.
Runs can differ: only generation, rather than the subsequent simulation, is seed-reproducible.

## Acceptance record — 2026-11-03

Windows PC: i9-13900K, RTX 4070 Ti, 64 GB RAM. Unreal 5.8.3, MSVC 14.44.35229,
SDK 10.0.22621.0, DirectX 12/SM6, Development package, wire 1.107, save schema 219.
Both measured scenes use 2560×1440, a five-second warmup, the HUD and embedded panels.

| Packaged check | Result |
| --- | --- |
| Create → render → save → load → quit | 32 people, 512² terrain, collision and 1,956 PCG grass instances restored; clean session marker removal |
| S1 terrain, 16.777 km² | 2,126 sampled frames; mean 5.650 ms, p99 6.970 ms |
| S1 local edits | Twenty alternating cut/fill responses; patch reader p99 3.752 ms; one changed tile per patch; current collision and grass checked, original surface restored |
| Running night village | 1,464 sampled frames; mean 6.834 ms, p99 8.785 ms; seven completed roofs and seven fuel-lit hearths |
| Construction capture | Day 45 at 07:24; 114 posts, nine wall panels, 16 scaffold instances and one actual carried load |

Both p99 budgets are 16.667 ms. S1 replays baseline/edited raster responses produced by
the real kernel: four cells, 51.2 m³ cut and 51.2 m³ fill. It uses the normal envelope decoder,
terrain reader, mesh updates and PCG surface, and tests actual collision traces and grass
transforms during the edits. This is an isolated renderer diagnostic, **not a lived digging
episode**; it does not alter or save the live kernel's world. The night scene is lived.
These sparse-village measurements do not establish later city-scale performance.

The kernel's full simcheck suite passes 1,391 tests (three intentional ignores), with
formatting and strict clippy clean. Commons checks, all ten smoke worlds, content validation,
schema freshness, the web build, 213 web unit tests and 30 browser tests pass (15 opt-in
demo tests skipped). Native citizen picking, Save, Load latest and Quit were also exercised
in the packaged application.

## Reproduce

```powershell
tools\build-unreal.ps1 -Package
tools\test-unreal.ps1
tools\test-unreal.ps1 -Scenario Spike
tools\test-unreal.ps1 -Scenario Night
tools\capture-unreal-demo.ps1 -FFmpeg <path-to-ffmpeg.exe>
```

The capture helper makes a fresh lived world, records through Unreal, saves and closes it,
then writes the source and titled videos here. Working frames and logs stay under ignored
`recordings/m2-capture`. To retitle the committed source without another simulation run:

```powershell
tools\capture-unreal-demo.ps1 -SourceVideo demos\m2\m2-first-light-raw.mp4 -FFmpeg <path-to-ffmpeg.exe>
```

Use `-TCEDemoDays=45 -TCEDemoHour=7 -TCEDemoOverview` with `-TCEDemo -TCEDemoRun
-TCEDemoExit` for a construction capture. Full controls, build instructions and rendering
limits are in the [client guide](../../unreal/README.md); the
[kit guide](../../art/vernacular/README.md) covers Blender and import validation.

## Earlier integration capture

[`integration-night.png`](integration-night.png) preserves the first Windows integration:
March 13 at 21:00, paused, primitive kit and early construction frames. Its editor run
measured mean 6.351 ms, p99 10.840 ms over 1,575 frames. It lacked the final water,
vegetation, kit, collision and LOD implementation, and retained a stale task caption.
It remains production history rather than the current M2 showcase.
