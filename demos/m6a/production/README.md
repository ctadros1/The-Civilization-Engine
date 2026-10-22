# Demo video production

This Remotion project adds a title card, explanatory chapter captions, and an end card to the M9
and M6a recordings and the observer captures from M0, M1, M3a–M3c, M4a–M4b and M5a. Final videos are
1920×1080 H.264. Original recordings remain unchanged in their milestone folders; copies used for
rendering are ignored under `public/`.

For a fresh capture, first build the web app and `civ-host`, then run the demo from `web/`:

```sh
TCE_DEMO=1 TCE_DEMO_VIDEO=1 TCE_SHOTS_DIR=../demos/m6a npm run test:e2e -- m6a-demo.spec.ts
```

Then copy the older source recordings and render from this folder:

```sh
npm ci
cp ../m6a-demo.webm public/m6a-source.webm
npm run dev -- --no-open
npm run render
```

For M3a–M5a, run the desired live demo captures from `web/`, then copy their raw recordings here
and render the titled versions. Captures are written to `demos/<milestone>/recording/`:

```sh
cp ../../m3a/recording/m3a-demo.webm public/m3a-source.webm
cp ../../m3b/recording/m3b-demo.webm public/m3b-source.webm
cp ../../m3c/recording/m3c-demo.webm public/m3c-source.webm
cp ../../m4a/recording/m4a-demo.webm public/m4a-source.webm
cp ../../m4b/recording/m4b-demo.webm public/m4b-source.webm
cp ../../m5a/recording/m5a-demo.webm public/m5a-source.webm
npm run render:milestones
```

`npm run render:archive` additionally refreshes the titled M0 and M1 videos and assumes their
sources have been copied from `assets/` into `public/`:

```sh
cp ../../../assets/m0/m0-demo.webm public/m0-source.webm
cp ../../../assets/m0/m0-recovery.webm public/m0-recovery-source.webm
cp ../../../assets/m1/m1-demo.webm public/m1-source.webm
npm run render:archive
```

Capture a milestone from `web/` with `TCE_DEMO=1 TCE_DEMO_VIDEO=1` and set
`TCE_SHOTS_DIR=../demos/<milestone>/recording`; the recording context starts after the simulation
has been prepared, at 1920×1080. Some milestone runs take several minutes. M5a lives thirty years
and can take much longer.

For the M9 showcase, the runner stores the raw recording and captures in `demos/m9/`. From this
folder, copy the raw observer recording into `public/` and render the titled video. The M9 edit
skips the initial loading and run-ahead dialog, then shows the household-resolution and climate
views in a 29-second 1920×1080 H.264 video:

```sh
cp ../../m9/m9-demo.webm public/m9-source.webm
npm run render:m9
npx remotion still src/index.ts M9ManySettlements ../../m9/m9-overview-poster.png --frame=210
```

Run these commands from this folder. `npm run dev -- --no-open` opens the Remotion preview.
The M0 recovery source is an early short capture of the recovery prompt; its edit explicitly says
that the recording ends before a successful reload is verified. The M1 and M6a captions describe
the recorded systems, not guaranteed outcomes in every world.
