# Web observer

The web observer (the M0 shell, and M1's people, families, fields and huts): TypeScript, Vite and PixiJS 8,
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
| `src/map/` | Terrain shading (`shade.ts`) and the PixiJS map with detail tiles, people, fields and huts (`view.ts`). |
| `src/people.ts`, `src/fields.ts`, `src/buildings.ts` | How people, fields and huts look on the map and what lies under a point: pure functions, unit tested. Huts are drawn from the shape the kernel expands, never designed here. |
| `src/format.ts` | Numbers, distances and simulated times in words. |
| `src/ui.ts`, `src/state.ts`, `src/main.ts` | Panels, dialogs, the store, and wiring. |
| `src/schema/generated/` | FlatBuffers TypeScript from `kernel/crates/civ-schema/schema/tce_wire.fbs`. Never edit; run `tools/gen-schema.sh`. |
| `e2e/` | Playwright specs; they start their own `civ-host` against a temporary saves folder. |

## Test hooks

`window.__TCE__` (mirroring Genesis's `window.__OBS__`) exposes plain-data state for tests and
debugging: `state()`, `map()`, `pointerAt(x, y)`, `panBy(dx, dy)`, `zoomBy(f)`, `fit()`,
`peopleOnScreen()`, `briefs()` (each person's id, sex and age) and `select(id)`.

## Recording the demo

`TCE_DEMO_VIDEO=1 npm run test:e2e` records a video of each spec into `test-results/`, and
`TCE_SHOTS_DIR=<dir>` saves step screenshots.
