# Web observer

The web observer (the M0 shell, M1's people, families, fields, huts, trails and god tool, M3a's tools, skills, market panel, workshops panel, land tenure and wealth panel, and M3b's knowledge panel): TypeScript, Vite and PixiJS 8,
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
| `src/market.ts` | What the market panel says about amounts, terms, payments and the price history: pure functions, unit tested. Terms, trades and money come from the kernel; the panel only words and draws them. |
| `src/firm.ts` | What the workshops panel says about a workshop's goods, holdings, monthly statements and hours: pure functions, unit tested. Records, wages and book lines come in words from the kernel. |
| `src/wealth.ts` | What the wealth panel says about a settlement's measures, its households and its yearly history, and the Gini chart's lines: pure functions, unit tested. The measures are the kernel's. |
| `src/knowledge.ts` | What the knowledge panel and the inspector say about who knows, is learning and has heard of each technique, and which techniques can be introduced to someone: pure functions, unit tested. States, sources and histories come in words from the kernel. |
| `src/ui.ts`, `src/state.ts`, `src/main.ts` | Panels, dialogs, the store, and wiring. |
| `src/schema/generated/` | FlatBuffers TypeScript from `kernel/crates/civ-schema/schema/tce_wire.fbs`. Never edit; run `tools/gen-schema.sh`. |
| `e2e/` | Playwright specs; they start their own `civ-host` against a temporary saves folder. The workshops spec builds its world with `cargo run --release -p civ-sim --example workshop_world`, so it needs cargo. |

## The market panel

Each settlement's market (M3a slice I), fetched with `GetMarkets` when the snapshot's
`markets_rev` changes (at most once a second): barter or the settlement's money, with the
kernel's summary; per good, what is offered and by how many households, the lowest terms asked in
each payment good (by the tool, or per 10 kg), what sold lately, what was wanted with none on
offer, its share of payments, the last terms, and a twelve-month line of what a unit was worth to
its sellers in hours of their own work; every household's and workshop's terms; and the latest trades in words.

## The workshops panel

Every household workshop (M3a slice J), fetched with `GetFirms` when the snapshot's `firms_rev`
changes (at most once a second): open ones first, then the closed with why they closed; what each
makes, who owns it, its record in words and the hours of work it would hire. A workshop's name
opens its page (`GetFirm`, read again as it changes): who set it up and since when its owners
have had it, its last sale, what it holds, its terms, its wage, its life in hours, a table of its
last twelve monthly statements and the latest lines of its books. Workshop names in the
chronicle and in the market's list of offers open the same page.

## Land tenure and the wealth panel

The new-world dialog's **Land tenure** lists the content's property regimes (M3a slice K), the
default chosen, with each one's description and its rules in sentences the kernel writes. The
world panel names the regime a world lives under and its rules, and the map's readout says who
holds the field under the pointer: its household, the settlement, or a holder that let it for a
share of its grain until a date.

The wealth panel, fetched with `GetWealth` when the snapshot's `wealth_rev` changes (at most
every two seconds; it changes monthly, when households form or end and when a year is
recorded), shows each settlement's measures as they stand: goods (in hours of work at its
prices), land worked, land held and floor area, each with its level and Gini; the richest
tenth's share of goods and who holds and works no land; a table and a chart of the Ginis at
each year's end; and its households, the most goods a head first, with what they hold, work,
let and rent.

## The knowledge panel

The knowledge panel (M3b slice M), fetched with `GetKnowledge` when the snapshot's
`knowledge_rev` changes (at most every two seconds; it changes when someone comes to know,
learns toward, hears of or loses a technique, and when people arrive, leave or die), lists each
settlement's techniques, those known there first. Each shows its state in the kernel's words
("known by 33", "known by one, Wren, aged 61; nobody learning", "lost in year 9 with Wren"). Open
it to see what a competent person can do with it and its prerequisites, who knows it (eldest
first), how many practised it in the last year, who is learning it, who has only heard of it,
and its history there. The inspector's **Knows** lists what someone knows, is learning (hours of
hours) and has heard of, and how each came. For the living it holds the god tool: choose a
technique they do not know, then **Teach** (they know it at once) or **Tell of it** (they only
hear of it). The inspector holds its redraw while that list has the focus.

## Controls

Besides Run and the speeds, **Run ahead…** lives a day, a month, a year, 5 or 10 years at full
detail as fast as the machine allows, shown as a task you can cancel; the world pauses when it
gets there and is autosaved. **Add a family** (on the map) sends a family where you next click:
it joins the nearest settlement within 600 m or camps there. Esc cancels. The list beside it
sends 5, 10 or 20 families together instead: they settle side by side around the click and all
join the settlement the first one joins or founds. Pointing at a hut, the readout gives its floor
area, how many it sleeps and how far its building has gone.

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
debugging: `state()` (with a summary of each market, workshop, settlement's wealth and knowledge, the workshop page open, and what the inspected person knows), `map()`, `pointerAt(x, y)`, `panBy(dx, dy)`, `zoomBy(f)`, `fit()`,
`peopleOnScreen()`, `briefs()` (each person's id, sex and age), `select(id)`, `openFirm(id)` and
`introduceTechnique(person, techniqueId, awareOnly)`.

## Recording the demo

`TCE_DEMO_VIDEO=1 npm run test:e2e` records a video of each spec into `test-results/`, and
`TCE_SHOTS_DIR=<dir>` saves step screenshots.

The M3a demo's pictures (`e2e/m3a-demo.spec.ts`: one seed under both property regimes, a village
of hundreds lived five years from the command line, then each world's wealth and market panels
and its village) run only when asked: `TCE_DEMO=1 TCE_SHOTS_DIR=<dir> npx playwright test
m3a-demo` (about 15 minutes; `TCE_DEMO_SEED`, `TCE_DEMO_BAND`, `TCE_DEMO_FAMILIES` and
`TCE_DEMO_DAYS` change the world).

The M1 demo (`e2e/demo.spec.ts`: ten years of a band, then save and reload) runs only when asked,
since it takes about four minutes: `TCE_DEMO=1 TCE_DEMO_VIDEO=1 npx playwright test demo`. It
prints `demo-mark` lines with the seconds at which the long run ahead begins and ends;
`assets/m1/m1-demo.webm` shows that stretch five times faster:

```sh
ffmpeg -i video.webm -filter_complex "[0:v]trim=0:51,setpts=PTS-STARTPTS[a];\
[0:v]trim=51:202,setpts=(PTS-STARTPTS)/5[b];[0:v]trim=202,setpts=PTS-STARTPTS[c];\
[a][b][c]concat=n=3:v=1:a=0,fps=20[out]" -map "[out]" -c:v libvpx-vp9 -crf 40 -b:v 0 m1-demo.webm
```
