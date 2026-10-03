# Web observer

The web observer (the M0 shell, and M1's people, families, fields, huts, trails and god tool): TypeScript, Vite and PixiJS 8,
without a UI framework. `civ-host serve` serves
the built files from `web/dist` and the observer socket at `/ws` from the same origin.

```sh
npm ci                 # once
npm run build          # type-check, then build dist/
npm test               # unit tests (vitest): envelope golden vectors, payloads, shading, calendar
npm run test:e2e       # browser tests (Playwright); needs dist/ and a built civ-host
npm run dev            # Vite on http://127.0.0.1:5173, forwarding /ws to civ-host on :7420
```

`npm run dev` expects a host already running (`civ-host serve`); set `TCE_HOST=host:port` to use
another port. `?host=127.0.0.1:7421` in the page URL connects to a host directly.

## Layout

| Path | What |
|---|---|
| `src/wire/envelope.ts` | The commons-wire envelope, hand-written; must match `commons/crates/commons-wire/tests/golden.json`. |
| `src/net/messages.ts` | Builds commands and queries, decodes the host's payloads. The only module that touches generated code. |
| `src/net/client.ts` | The WebSocket: handshake, correlation ids, reconnect with backoff. |
| `src/map/` | Terrain shading (`shade.ts`) and the PixiJS map with detail tiles, people, fields, huts, worn ground and trails (`view.ts`). |
| `src/people.ts`, `src/fields.ts`, `src/buildings.ts`, `src/paths.ts` | How people, fields, huts and paths look on the map and what lies under a point: pure functions, unit tested. Huts are drawn from the shape the kernel expands and trails from the lines it traces, never designed here. |
| `src/format.ts` | Numbers, distances and simulated times in words. |
| `src/ui.ts`, `src/state.ts`, `src/main.ts` | Panels, dialogs, the store, and wiring. |
| `src/schema/generated/` | FlatBuffers TypeScript from `kernel/crates/civ-schema/schema/tce_wire.fbs`. Never edit; run `tools/gen-schema.sh`. |
| `e2e/` | Playwright specs; they start their own `civ-host` against a temporary saves folder. |

## Controls

Besides Run and the speeds, **Run ahead…** lives a day, a month, a year, 5 or 10 years at full
detail as fast as the machine allows, shown as a task you can cancel; the world pauses when it
gets there and is autosaved. **Add a family** (on the map) sends a family where you next click:
it joins the nearest settlement within 600 m or camps there. Esc cancels.

## The panels alone

`?view=panels` shows the panels without the map, for a host that draws the world itself: Unreal's
WebBrowser widget opens the address the kernel library's `panel_url` gives (ADR-0005 §6), which
carries a `?token=` the page passes to its socket. Without a map, showing a place (a settlement
or person link) dispatches a `tce:focus` event on `window` whose `detail` is `{ x, y }` in metres
east and south, for the host to move its camera; the host can open someone in the inspector with
`window.__TCE__.select(id)`. The socket refuses pages from other sites, so a page must be served
from this machine.

## Test hooks

`window.__TCE__` (mirroring Genesis's `window.__OBS__`) exposes plain-data state for tests and
debugging: `state()`, `map()`, `pointerAt(x, y)`, `panBy(dx, dy)`, `zoomBy(f)`, `fit()`,
`peopleOnScreen()`, `briefs()` (each person's id, sex and age) and `select(id)`.

## Recording the demo

`TCE_DEMO_VIDEO=1 npm run test:e2e` records a video of each spec into `test-results/`, and
`TCE_SHOTS_DIR=<dir>` saves step screenshots.

The M1 demo (`e2e/demo.spec.ts`: ten years of a band, then save and reload) runs only when asked,
since it takes about four minutes: `TCE_DEMO=1 TCE_DEMO_VIDEO=1 npx playwright test demo`. It
prints `demo-mark` lines with the seconds at which the long run ahead begins and ends;
`assets/m1/m1-demo.webm` shows that stretch five times faster:

```sh
ffmpeg -i video.webm -filter_complex "[0:v]trim=0:51,setpts=PTS-STARTPTS[a];\
[0:v]trim=51:202,setpts=(PTS-STARTPTS)/5[b];[0:v]trim=202,setpts=PTS-STARTPTS[c];\
[a][b][c]concat=n=3:v=1:a=0,fps=20[out]" -map "[out]" -c:v libvpx-vp9 -crf 40 -b:v 0 m1-demo.webm
```
