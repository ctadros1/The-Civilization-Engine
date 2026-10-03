# engine-commons (staged)

Code shared between The Civilization Engine, Prometheus and Genesis ([plan §3.3](../PROJECT_PLAN.md#33-shared-code-with-prometheus-and-genesis-engine-commons)).
It lives here, as its own Cargo workspace, until a second engine adopts it. Extracting it into
`ctadros1/Engine-Commons` is then a directory move (`git subtree split --prefix=commons`), after
which engines pin it by tag.

**Admission rule:** code enters only if at least two engines use it, or will within one milestone.
It must contain no engine-specific concepts: no "agent", "settlement" or "organism" types.

| Package | Status | Contents |
|---|---|---|
| [`commons-wire`](crates/commons-wire) | **Implemented (v0, M0)** | Frame envelope, frame kinds and delivery contracts, schema versioning, sequencer. [ADR-0001](../decisions/0001-boundary-schema.md) |
| [`commons-persist`](crates/commons-persist) | **Implemented (v0, M0)** | Chunked snapshot container, crash-safe publication, generation directories. [ADR-0002](../decisions/0002-snapshot-container.md) |
| `commons-wire` transports | Planned (M2) | FFI ring buffer, recording file. The WebSocket transport currently lives in `civ-host`. |
| `commons-rng` | Planned | Keyed deterministic RNG; needed by Prometheus and Genesis, not by TCE. |
| `commons-record` | Planned | Recording format for frame streams a viewer can play back. |
| `EngineBridge` (UE plugin) | Planned (M2) | DLL hosting, triple-buffer frame reader, trip interpolation, panel host. |
| `web-kit` (TS) | Planned (M1–M2) | Inspector, timeline, chart components and the `commons-wire` client. The envelope decoder currently lives in `web/src/wire/`. |

## Build and test

```sh
cd commons
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
