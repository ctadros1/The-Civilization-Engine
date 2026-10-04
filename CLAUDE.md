# Claude Session Guide

`AGENTS.md` is the operating contract; this page is the short version.

## Session start

1. Read `README.md` (status), `AGENTS.md`, and the current milestone in `PROJECT_PLAN.md` §7.
2. Check `PROJECT_PLAN.md` §9 (decisions) and the ADRs in `decisions/` before changing the
   boundary schema, saves or anything else expensive to reverse.
3. Read the research reports for the area you are working on before designing (see
   `research/README.md`).
4. Run the relevant tests before editing.

## Non-negotiables

- The engine authors the vocabulary, never the plot: no scripted events, eras or outcomes.
- Every milestone stays usable: launch, create, watch, save, load, quit, with no data loss.
- The kernel owns truth; the web observer only shows state and forwards commands.
- Plausible, not proven: no research campaigns; time-box emergence and log `NUDGE:` entries.
- ADRs only for expensive-to-reverse decisions (about three per milestone at most).
- Never claim something works without running it.

## Everyday commands

```sh
tools/run.sh                     # or tools\run.ps1 on Windows: build if needed, serve, open
cd kernel && cargo test --workspace --profile simcheck
cd kernel && cargo clippy --workspace --all-targets -- -D warnings
cd kernel && cargo run --release -p civ-host -- smoke
cd web && npm run build && npm test && npm run test:e2e
tools/gen-schema.sh --check      # after any schema change (needs flatc 25.12.19)
```
