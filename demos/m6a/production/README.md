# M6a demo video

This Remotion project adds a title card, chapter captions, and an end card to the 1920×1080
observer recording in `../m6a-demo.webm`. The raw capture is copied to `public/m6a-source.webm`
for local preview and rendering; it is ignored by Git so the source is not duplicated in the repo.

For a fresh capture, first build the web app and `civ-host`, then run the demo from `web/`:

```sh
TCE_DEMO=1 TCE_DEMO_VIDEO=1 TCE_SHOTS_DIR=../demos/m6a npm run test:e2e -- m6a-demo.spec.ts
```

Then render from this folder:

```sh
npm ci
cp ../m6a-demo.webm public/m6a-source.webm
npm run dev -- --no-open
npm run render
```

Run the first block from `web/`; run the second block from this folder. The render writes
`../m6a-overview.mp4`. `npm run dev` opens the Remotion preview. The recorded outcome varies by
world; the captions describe the systems shown rather than promise a particular result.
