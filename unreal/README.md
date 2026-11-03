# Unreal observer

The Windows M2 client runs on Unreal **5.8.3**, MSVC **14.44.35229**
(Visual Studio Build Tools 2022 17.14), Windows SDK **10.0.22621.0**, and the repository's
pinned Rust **1.97.0** MSVC toolchain. Engine binaries are installed at
`D:\Program Files\Epic Games\UE_5.8` on the development PC.

From the repository root:

```powershell
tools\build-unreal.ps1 -Package
tools\run-unreal.ps1          # prefers the packaged executable
tools\run-unreal.ps1 -Editor  # editor in game mode
```

Pass `-Engine <directory>` to use another installation of the pinned engine. The development
PC also has `C:\TCE`, a directory junction to this checkout, to keep compiler and cooking paths
short. It contains the same files, not a second checkout.

The packaged application is `dist\Unreal\Windows\TCE.exe`; it runs without the editor.
Keep the entire `Windows` directory together. The package includes the kernel DLL, content
packs, web panels, and cooked kit. Saves are under the application's `TCE\Saved\Worlds`.
Editor game sessions use `unreal\Saved\Worlds`. Closing normally joins the kernel's threads
and saves a changed world; the two launch modes keep separate save folders.

`tools/test-unreal.ps1` checks the packaged executable end to end, including creating,
saving, loading and closing a world, runtime terrain collision and PCG ground cover.
Each run uses its own disposable save folder under `recordings/m2-checks`.

## Controls

The toolbar offers a new valley, pause/resume, clock speeds, following the selected resident,
save, loading the latest compatible save, and quit. The embedded web panels offer the
existing inspectors and world/session controls. Click the world before using shortcuts:

| Key | Action |
| --- | --- |
| N | New 512-cell valley, seed 12, 32-person band |
| Space | Pause/resume |
| 1 / 2 / 3 | 1× / 10× / 600× |
| W A S D | Move the camera |
| Mouse wheel | Zoom |
| Click a citizen | Open their inspector |
| F | Follow/stop following the selected resident (first resident when none is selected) |
| Q / E | Orbit the camera |
| Ctrl+S | Save |
| L | Load the latest compatible save |
| Tab | Show/hide the panels |

The shared plugin is in `commons/unreal/EngineBridge`. It negotiates ABI and wire versions,
loads one DLL for the process lifetime, owns copied buffers, drains ordered notifications,
and takes the newest snapshot. Rust alone owns simulation, content, and saves. The observer
reads generated FlatBuffers types. Wire **1.107** appends exact grammar parts to the
building query; saves remain at schema **219**. The client requires wire 1.107 or later.
Rust's latest-value snapshot channel and caller-owned copies implement ADR-0005's
replacement for the original literal triple-buffer proposal. Slow drawing can skip a
snapshot; ordered commands, replies and notifications are retained.

Terrain uses separate 64-cell procedural-mesh tiles, CPU detail levels, skirts and
camera-local asynchronous collision. Local earthwork revisions retain unchanged tiles.
`UKernelTerrainSurface` exposes the same elevation field to PCG's CPU Surface Sampler;
its cosmetic grass excludes water, plots and fields. It creates no simulation resources.
Water uses the kernel's lake levels, sea level and river reach widths/centrelines.
Field surfaces follow the ground. Terrain shadow casting is disabled to avoid non-Nanite
VSM page overflow; the kit uses Nanite, Lumen and VSM. Ground shading is procedural;
runtime virtual textures are not needed by this first kit.

Citizens have instanced bodies, heads, limbs and actual carried-load bundles. Shared timed
paths drive their position; paused and Accelerated clocks do not extrapolate. The
[Blender kit](../art/vernacular/README.md) is authored offline and imported by
`Tools/build_assets.py`. Exact kernel parts supply dimensions, orientation and stages.
Pieces appear as labor progresses, with partial thatch and temporary working platforms.
Hearth light requires a built hearth and household fuel. UMG owns the native toolbar and
embedded web panels. Panel locate events move the native camera; picking a person opens
their existing inspector. Motion blur is disabled to keep a moving follow view readable.

## Reproduce the acceptance checks

```powershell
tools\test-unreal.ps1                 # create, draw, save, load, close
tools\test-unreal.ps1 -Scenario Spike # kernel-produced cut/fill raster replay
tools\test-unreal.ps1 -Scenario Night # running night village at 2560×1440
tools\capture-unreal-demo.ps1        # needs FFmpeg; captures and titles the video
```

The spike produces baseline and edited raster responses through the real kernel, with
four cut/fill cells. The packaged client replays twenty alternating responses through its
normal frame decoder and terrain reader. It checks changed-tile count, the displayed
height, the PCG elevation, actual collision traces, and grass transforms over the edited
patch, then restores the original surface. It fails above a 16.667 ms p99 frame or patch
reader budget. This is a renderer diagnostic; it does not pretend that a household dug
the fixture, and never saves the altered presentation as kernel state.

`-TCEDemo` creates seed 12, 512 cells at 8 m (16.78 km²), and 32 founders; it advances
naturally for 90 days to 21:00 and follows a resident. `-TCEDemoRun` resumes at 1×;
`-TCEDemoExit` closes after capture/save. `-TCEDemoDays`, `-TCEDemoHour`,
`-TCEDemoOverview` and `-TCEDemoImage` support construction/daytime captures.
`-TCEDemoFrames=<folder>` records native JPEG frames for the video. Recording readbacks
are deliberately measured separately from the performance gate.

The first kit and crowd v0 target the M1 village. Later building styles, skeletal close-ups,
seasons/weather visuals and fake interiors remain later work. Worlds above one million
raster samples use a downsampled overview, rather than promising full-resolution terrain
across every offered world size. Village measurements do not establish city-scale performance.
