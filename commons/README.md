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
| [`cpp`](cpp) | **Implemented (M2)** | Header-only C++17 for C++ hosts such as the Unreal plugin: `commons_wire.hpp`, the frame envelope, tested against the same golden vectors as Rust and TypeScript; `timed_path.hpp`, positions along timed paths (trip interpolation); and the FlatBuffers C++ runtime in `third_party/`. Built and tested with the system compiler by [`crates/commons-cpp`](crates/commons-cpp) |
| `commons-wire` transports | Planned | Recording file. The WebSocket transport lives in `civ-host`; TCE's in-process transport is its C interface (`civ-ffi`, ADR-0005). |
| `commons-rng` | Planned | Keyed deterministic RNG; needed by Prometheus and Genesis, not by TCE. |
| `commons-record` | Planned | Recording format for frame streams a viewer can play back. |
| `EngineBridge` (UE plugin) | Planned (M2, on the Windows PC) | DLL hosting, frame reading, trip interpolation and the panel host inside Unreal; its engine-agnostic C++ core is in `cpp/`. |
| `web-kit` (TS) | Planned (M1–M2) | Inspector, timeline, chart components and the `commons-wire` client. The envelope decoder currently lives in `web/src/wire/`. |

## Build and test

```sh
cd commons
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
