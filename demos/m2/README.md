# First Windows Unreal integration

`integration-night.png` is an unedited 2560×1440 capture of the first Unreal client on
2026-11-03. Seed 12, 512 cells per side at 8 m per cell (16.78 km²), 32 people, March 13
at 21:00. It shows the kernel's construction frames and simple citizen instances, the UMG
toolbar, and the existing web panels. The image is intentionally kept as evidence of the
early integration; it is not the finished M2 visual demo.

The editor game-mode capture measured mean frame time **6.351 ms**, p99 **10.840 ms**,
over 1,575 frames following a five-second warmup. The world was paused, the camera followed
one resident, and the current sparse scene had no water, vegetation, or finished building kit.
This does not establish performance while editing terrain or at later city scales. The
capture's status caption still shows the last run-ahead task; that is a known UI defect.

Regenerate with `tools/build-unreal.ps1 -Package`, then launch the executable with `-TCEDemo`.
It creates a new test valley, advances twelve days to 21:00, measures frames and requests a
screenshot and save. Captures are written to `TCE/Saved/Screenshots/Windows` in a package,
or `unreal/Saved/Screenshots/WindowsEditor` in editor game mode. It remains open for review.

The full S1/M2 acceptance gates are tracked in [the client guide](../../unreal/README.md).
