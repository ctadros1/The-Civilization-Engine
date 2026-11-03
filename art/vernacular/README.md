# First vernacular kit

The M2 hut vocabulary is built offline with **Blender 5.2.3 LTS**. The source
`build_kit.py` writes `vernacular-kit.blend`, nine FBX modules, and `kit.json`.
It creates reusable parts, never a settlement or a complete house.

```powershell
& 'C:\Program Files\Blender Foundation\Blender 5.2\blender.exe' --background --python art\vernacular\build_kit.py
tools\build-unreal.ps1 -Package
```

| Module | Local axes and use |
| --- | --- |
| TimberPost | One metre along Z; hewn post, scaled to the kernel part |
| TimberBeam | One metre along X; beam, rafter and temporary scaffold |
| WattlePanel | One metre along X/Z; woven infill, Y is thickness |
| DaubPanel | Centred one-metre box; earth wall infill |
| ThatchCone | Nominal one-metre diameter/height; twelve overlapping courses and a thin outer skin over the supporting rafters |
| ThatchWedge | One sixteenth of that cone, sharing its centre pivot |
| EarthFloor | One-metre diameter; thin beaten-earth surface |
| HearthBowl | One-metre diameter/height; scaled to the hearth's dimensions |
| Plank | Centred one-metre box; reserved kit primitive |

The manifest records nominal metres, pivots and mesh counts. FBX export uses X forward,
Z up, applied transforms, explicit units and face normals. The pinned Unreal project
disables Interchange FBX routing and imports through `FbxFactory` with textures,
materials and generated collision disabled. `unreal/Tools/build_assets.py` checks
module bounds, records the source SHA256 and importer revision, assigns named material
slots, enables Nanite, and cooks the resulting assets. Build output is disposable;
the blend, FBX, manifest and imported assets are tracked with Git LFS where applicable.

Runtime assembly consumes the kernel's exact centimetre parts, their orientation,
stage and progress (wire 1.107). Completed stages remain; the current stage reveals
work-sized pieces. Roof wedges expose partial thatching. Working platforms follow the
active wall/roof stage. A carried bundle appears only for an actual carried load;
night hearth light requires a built hearth and household fuel. These are presentations
of kernel state and never change stocks, labor, navigation or saves.
