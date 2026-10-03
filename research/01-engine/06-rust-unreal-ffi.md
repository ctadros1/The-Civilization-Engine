# Engineering report: embedding TCE’s Rust kernel in Unreal Engine 5.8 on Windows

**Recommendation:** build a small, conventional C++ **Runtime plugin** that explicitly loads the Rust `cdylib` through a **versioned C function table**. Rust should own the authoritative world and its worker threads; Unreal should own presentation objects. Exchange batched commands and immutable snapshots, initially copying snapshots into Unreal-owned memory. Keep the DLL loaded for the lifetime of a packaged game, and treat editor DLL replacement as a separate, carefully controlled feature.

This avoids making TCE depend on a comprehensive Rust-to-Unreal binding framework. The main engineering risks are not calling Rust functions: they are **thread shutdown, memory lifetime, packaging, and maintaining a reliable interface while both sides evolve**.

**Version baseline:** Epic released UE 5.8 on June 23, 2026. The Rust standard-library documentation retrieved for this report identifies itself as **Rust 1.98.1**. Recommendations below target Windows x64 and distinguish documented behavior from proposed TCE policies. I did not execute a UE 5.8 Windows build or find a controlled benchmark of this exact architecture at TCE’s scale. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

---

## 1. Options and trade-offs

| Technique | How it works | Advantages | Costs and suitability for TCE |
| --- | --- | --- | --- |
| **Explicitly loaded `cdylib`, C ABI** | Load the DLL at runtime, resolve one entry point, obtain a function table. | Small dependency surface; explicit compatibility checks; independent Rust builds; controlled replacement. | You must implement ownership and lifecycle correctly. **Best fit.** |
| **Import-linked or delay-loaded `cdylib`** | Link an import library; optionally defer loading until the first call. | Convenient ordinary function calls; conventional native integration. | Resolved import-table entries complicate DLL replacement. Suitable when the DLL never reloads. |
| **Rust `staticlib`** | Link Rust code into the UE module or executable. | Fewer runtime deployment components; no independently managed DLL lifetime. | Rust changes require relinking the host; no independent DLL replacement. Valid alternative, but contrary to TCE’s chosen deployment model. |
| **Generated C++ bridge, such as CXX** | Generate mutually checked Rust/C++ bindings and type conversions. | Useful for richer interfaces; reduces handwritten glue. | Does not solve UObject ownership, thread affinity, or unloading. More machinery than a small simulation interface needs. |
| **Comprehensive Unreal–Rust framework** | Bind gameplay classes, reflection, engine APIs, and editor behavior. | Convenient when most gameplay is written against Unreal from Rust. | Much larger engine-version maintenance burden. Existing examples reviewed here carry production-readiness warnings. |
| **Separate simulation process** | UE communicates with a Rust executable through IPC or shared memory. | Stronger crash isolation; replacement does not unload code from Unreal’s process. | Additional protocol, process supervision, and synchronization complexity. Worth retaining as a future option, not the initial implementation. |

Rust documents `cdylib` and `staticlib` as foreign-language integration outputs; Epic documents explicit and delay-loaded DLL integration. CXX addresses language interoperability rather than engine lifecycle. The performance and maintenance assessments above are architectural judgments, not comparative TCE measurements. [Rust Documentation](https://doc.rust-lang.org/reference/linkage.html)

### Why a small bridge is particularly appropriate here

TCE does not need Rust to implement `AActor`, participate in Unreal reflection, or call rendering APIs. It needs a simulation service with a limited contract: create a world, submit commands, advance independently, publish observations, save, and stop.

I would maintain four components:

| Component | Responsibility |
| --- | --- |
| `tce_core` | Simulation, persistence, and tests; no Unreal dependency. |
| `tce_ffi` | C ABI, validation, command queues, snapshots, and thread lifecycle. |
| `TCERuntime` | DLL loading, game-thread integration, and presentation. |
| `TCEEditor` | Optional rebuild/reload commands, diagnostics, and editor-only UI. |

A headless executable using `tce_core` gives you a fast test and profiling environment without changing the production architecture.

---

## 2. Build integration, loading, and packaging

### 2.1 Use the MSVC Rust target

Build the kernel for **`x86_64-pc-windows-msvc`**. Rust documents this as a Tier 1 target with the Microsoft x64 C ABI and native Windows object/debug formats. Use the MSVC toolchain and Windows SDK supported by your installed UE 5.8 build, and pin that environment for reproducible builds. Do not select MinGW merely because another Rust project uses it. [Rust Documentation](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html)

For the FFI crate:

```
# tce_ffi/Cargo.toml
[lib]
name = "tce_kernel"
crate-type = ["cdylib"]
```

At the Cargo workspace root:

```
[profile.release]
debug = 2
strip = "none"
panic = "unwind"
```

This preserves full debugging information while retaining optimization and unwind-based panic handling. Evaluate ThinLTO separately; it is a build-time/performance trade-off, not a correctness requirement. An optimized development profile is also useful: an unoptimized simulation can give misleading impressions of scalability. [Rust Documentation](https://doc.rust-lang.org/cargo/reference/profiles.html)

### 2.2 CRT compatibility: match conventions, but keep allocators separate

For the initial configuration, use the dynamic CRT on the Rust side, explicitly selecting `-C target-feature=-crt-static` where appropriate. Ensure native C/C++ dependencies built by Cargo use compatible runtime settings. Rust documents CRT selection through `crt-static`; Unreal normally uses the release CRT even in Debug configurations unless `bDebugBuildsActuallyUseDebugCRT` is enabled. **Cargo debug information and the Microsoft Debug CRT are different settings.** [Rust Documentation](https://doc.rust-lang.org/reference/linkage.html)

An identical compiler minor version is not intrinsically required for a carefully defined C interface containing fixed-layout values and opaque pointers. Nevertheless, using one pinned, Epic-supported native toolchain reduces avoidable differences.

More importantly, **matching CRT settings does not make Unreal’s allocator and Rust’s allocator interchangeable**. Microsoft documents the hazards of transferring CRT-owned objects and freeing memory across incompatible boundaries. Keep allocation and deallocation paired within the owning implementation. [Microsoft Learn](https://learn.microsoft.com/en-us/cpp/c-runtime-library/potential-errors-passing-crt-objects-across-dll-boundaries?view=msvc-170)

### 2.3 Invoke Cargo through an explicit build step—not incidental rules evaluation

My preferred build pipeline is:

**Generate/verify the C header → build Rust → publish the selected DLL and manifest → run UBT/UAT.**

Use one idempotent PowerShell script or small build driver for local builds and CI. Pin `rust-toolchain.toml`, commit `Cargo.lock`, use `--locked`, and pin the header generator. Maintain distinct output locations for configurations and concurrent builds.

For IDE convenience, invoke the same driver through a target/plugin pre-build hook. Epic exposes plugin `PreBuildSteps`, and its module documentation explicitly refers to libraries built through `TargetRules.PreBuildSteps` or `PreBuildTargets`. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Projects/FPluginDescriptor)

**Avoid invoking `cargo` as an uncontrolled side effect inside a `ModuleRules` constructor.** My reason is build predictability: describing dependencies and executing a separate toolchain should not be accidentally coupled to rules evaluation.

Also, `ExternalDependencies` is not a substitute for Cargo integration. Epic defines it as a list of files whose changes invalidate the UBT makefile; it does not describe it as an external build system. Use it for relevant manifests/generated inputs, while the build driver remains responsible for rebuilding Rust. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/module-properties-in-unreal-engine)

For clean builds, make sure header generation happens before C++ dependency discovery. Update generated headers only when their contents change, so a Rust implementation-only edit does not needlessly recompile the C++ bridge.

### 2.4 Generate headers with cbindgen

Use **cbindgen** to generate the C-facing declarations from the narrow FFI crate. It generates C or C++ headers from Rust declarations and is used in Mozilla’s interoperability tooling. It does not design a stable ABI for you. [GitHub](https://github.com/mozilla/cbindgen)

Keep a checked-in generated header or a deterministic generation step, and have CI reject unexplained header drift. Test layout explicitly: size, alignment, and field offsets on both sides.

My suggested compatibility scheme has separate identifiers for:

* **ABI version:** function signatures and in-memory interface layouts.
* **Save schema version:** persistent world representation.
* **Kernel build ID:** exact code and symbols.

These must not be treated as interchangeable.

### 2.5 Load explicitly and negotiate compatibility

In packaged builds, resolve the plugin’s installed directory and construct an absolute DLL path. Load through `FPlatformProcess::GetDllHandle`; it includes Unreal-specific dependency resolution and diagnostic logging. Windows dependency resolution still matters even when the top-level DLL path is explicit. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine)

Resolve one export, for example:

```
/* Proposed interface shape, not a complete header. */
uint32_t tce_get_api(
    uint32_t requested_abi_major,
    uint32_t out_api_bytes,
    TceApiV1* out_api);
```

The plugin uses `FPlatformProcess::GetDllExport` to obtain it, then checks the negotiated ABI, structure sizes, required functions, and build metadata before creating any world.

I recommend a **host-owned copy of the function table**, with an explicit “invalid” state during transitions. Never leave half of the old table installed after a failed reload.

For this explicit-binding design, do not also import-link the same kernel functions. That would create a second, less visible route to the DLL.

### 2.6 Stage the DLL as a loose runtime dependency

Inside the Runtime module’s Win64 build rules, the central staging declaration can be as simple as:

```
RuntimeDependencies.Add(
    Path.Combine(PluginDirectory, "Binaries", "Win64", "tce_kernel.dll"),
    StagedFileType.NonUFS);
```

The build driver must already have deployed the selected artifact there. Epic documents runtime dependency staging and distinguishes loose `NonUFS` files from packaged assets. A native DLL must be available to the operating-system loader; putting it only inside a pak/IoStore container is not sufficient. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine)

My packaging policy would additionally require that the loader reside in a **Runtime**, not Editor-only, module; all non-system DLL dependencies be staged; and the packaged executable contain no dependency on Cargo, Visual Studio, a source checkout, or developer `PATH` entries.

The decisive test is a **packaged Shipping build on a clean Windows machine**, not successful Play In Editor.

### 2.7 Preserve PDBs as release artifacts

On Windows MSVC targets, Rust’s packed debug information uses PDB files. Preserve the exact matching DLL/PDB pair, source revision, and build manifest for every distributed build. Renaming a DLL for editor replacement does not eliminate the need to retain its matching symbols. [Rust Documentation](https://doc.rust-lang.org/rustc/codegen-options/index.html)

Keep symbols in a private archive or symbol service. Epic’s `DebugNonUFS` staging category is available when debug files should accompany a particular package, but distributing PDBs to every player is not required for your internal symbolication workflow. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine)

---

## 3. Threading and communication architecture

### 3.1 Let Rust own the simulation’s execution lifetime

For TCE, I recommend one Rust coordinator thread and a configurable, dedicated worker pool. The coordinator handles simulation scheduling, commands, snapshot publication, and orderly shutdown.

A C++ `FRunnable` is a reasonable alternative when you want Unreal to own the outer thread. However, it does not remove the need to manage any Rust worker threads or their termination. I would not put a permanent simulation loop into a general-purpose Unreal task: model persistent execution explicitly, and use task systems for finite work.

The ownership boundary should be strict:

**Rust owns the world. Unreal receives observations and sends requests. Neither side reaches into the other’s mutable object graph.**

Unreal’s threading documentation explains why this matters: game-thread objects and rendering representations have different ownership and lifetimes, and the render thread may lag behind the game thread. Rust workers should therefore not read or mutate ordinary UObject state. [dev.epicgames.com](https://dev.epicgames.com/documentation/en-us/unreal-engine/threaded-rendering-in-unreal-engine)

### 3.2 Rayon is suitable, but shutdown needs special treatment

Use a **local Rayon pool**, not the global pool. Rayon documents that its global pool does not terminate until process exit, and that dropping a pool is not itself a guarantee that its worker threads have finished. Its custom spawn handler explicitly places responsibility for waiting for termination on the caller. [Docs.rs](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html)

For an unloadable Windows DLL, I would retain ordinary `std::thread::JoinHandle`s for all workers through a custom spawn handler. Shutdown should stop scheduling, finish or cancel structured work, drop the pool, and then join every worker.

There is a subtle distinction here:

* Rust’s `thread::scope` documentation warns that scope completion can precede completion of thread-local destructors.
* Ordinary `JoinHandle::join` explicitly waits for those destructors too.
* `is_finished()` can become true before the native thread has completely stopped. [Rust Documentation](https://doc.rust-lang.org/std/thread/fn.scope.html)

Consequently, **“the pool was dropped,” “the scope returned,” and “all counters are zero” are not interchangeable with “all threads are safe to unload.”**

Handle partial initialization and panics through the same owned shutdown machinery. Avoid detached jobs or dependencies that quietly create a second, process-global executor.

### 3.3 Use two different communication policies

I recommend separating traffic by semantics.

| Traffic | Policy |
| --- | --- |
| User commands, save requests, authoritative notifications | Sequenced, acknowledged, bounded, and not silently discarded. |
| Presentation snapshots | Latest complete value; intermediate snapshots may be skipped. |
| Logs and diagnostics | Bounded queue with an explicit overflow policy. |

For example, a rendering frame can skip an intermediate position snapshot. It must not accidentally erase a command to enact a law or lose the only notification that a settlement was destroyed.

Initially, publish complete presentation snapshots or sufficiently complete regional snapshots. Delta-only streams require additional baseline recovery when the consumer falls behind.

### 3.4 Triple buffering: keep the synchronization entirely inside Rust

A triple buffer is a good match for **one simulation producer and one game-thread consumer**. The reviewed `triple_buffer` crate implements a single-producer/single-consumer latest-value exchange; it is not a general multi-reader ownership system. [Docs.rs](https://docs.rs/triple_buffer/latest/triple_buffer/)

For TCE, use three reusable Rust-side buffers: a writable buffer, a consumer-held immutable buffer, and an exchange buffer. Once published, a buffer must not be mutated until ownership returns to the producer.

My recommended FFI operation is:

**`try_copy_snapshot(handle, caller_buffer, capacity, out_metadata)`**

It selects a complete available snapshot and copies it into preallocated Unreal-owned memory. It should return promptly with “no new snapshot” or “buffer too small” rather than wait for a simulation tick.

This deliberately avoids exposing cross-language atomics or long-lived raw Rust pointers. Start with a proven exchange implementation or a tiny locked ownership swap; do not begin with hand-written lock-free synchronization.

For insufficient capacity, return the required size without partial output. On retry, permit a newer complete snapshot and report its sequence number, rather than pretending the earlier size query pinned a particular version.

### 3.5 The render thread is not the third triple-buffer owner

This is a particularly important pitfall.

After copying a snapshot, the game thread can build Unreal-owned presentation data and submit it through appropriate rendering APIs. Any render command must retain its own data until execution completes. Epic’s rendering guidance emphasizes this separation and provides render-command fences for lifetime coordination. [dev.epicgames.com](https://dev.epicgames.com/documentation/en-us/unreal-engine/threaded-rendering-in-unreal-engine)

Do **not** give the renderer a pointer to the Rust buffer and then release that buffer at the end of the game tick.

Similarly, interpolation usually needs two historical snapshots. Retain those as Unreal-owned data rather than holding multiple slots from a single-consumer triple buffer indefinitely.

Avoid `FlushRenderingCommands` every frame. With the copying design, DLL shutdown should not need to reason about GPU access to Rust allocations because no such access exists.

### 3.6 Separate simulation time from presentation time

My initial scheduling policy would be:

| Setting | Initial TCE choice—not a measured optimum |
| --- | --- |
| Display | 60 fps |
| Snapshot publication | Around 20 Hz at ordinary observation speeds |
| Simulation advancement | Its own fixed internal scheduling rules |
| Rust workers | Configurable; begin testing around 6–8 workers |
| Performance sweep | Compare 4, 6, 8, and 12 workers under realistic rendering load |

The exact i9 model was not supplied, so do not hard-code a core topology. Optimize for simulation throughput **subject to game/render frame-time limits**, not for saturating every logical CPU.

Include simulation time, snapshot sequence, and a world/reload epoch in each snapshot. Interpolate against an intentional presentation delay; when data is late, follow an explicit hold/extrapolation policy instead of blocking the game thread.

---

## 4. FFI ownership, errors, and crash reporting

### 4.1 Make the interface smaller than the implementation

The function table should expose operations resembling:

`create`, `start`, `submit_commands`, `try_copy_snapshot`, `poll_status`, `request_save`, `request_stop`, `join`, and `destroy`.

Document which may run concurrently and which require exclusive access. In particular, destruction must not race any other operation.

An opaque handle should represent a thread-safe control object containing channels and lifecycle state—not an excuse to create a new unrestricted `&mut World` whenever Unreal calls into Rust.

Use fixed-width integers, explicit-length byte strings, and narrowly defined C-layout structures. Rust’s default representation is not a stable C layout, and `repr(C)` on an outer structure does not automatically change the representation of nested types. Apply and test layout rules throughout the exposed schema. [Rust Documentation](https://doc.rust-lang.org/reference/type-layout.html)

I would exclude `Vec`, `String`, trait objects, Rust references, C++ containers, `FString`, `FVector`, and `FTransform` from the ABI. Convert explicitly, including units, coordinate conventions, and precision.

Prefer integer status codes to accepting arbitrary Rust enum discriminants from C++. Treat nullability, lengths, alignment, and pointer lifetime as contractual obligations: an FFI wrapper cannot reliably turn an arbitrary dangling pointer into a recoverable error. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

### 4.2 Allocation rules should be unambiguous

My preferred rules are:

**Inputs are borrowed only for the duration of the call.** A queued command must copy any referenced bytes before returning.

**Output storage belongs to the caller.** Rust writes into caller-provided buffers but never frees them.

**Opaque Rust objects are destroyed through the same loaded kernel instance that created them.** No `FMemory::Free` on Rust allocations, and no Rust deallocation of Unreal memory.

A zero-copy acquired snapshot can be added later, but it must carry a release obligation that prevents buffer reuse and DLL unloading. The copying design avoids that complexity initially.

### 4.3 Catch panics inside Rust, then fault the kernel

Use `panic = "unwind"` and catch panics **before they leave any exported ABI function** that can panic. Cover the coordinator and background execution paths too; a catcher around a game-thread call does not catch a panic occurring elsewhere.

Rust documents that `catch_unwind` catches unwinding panics, not aborting panics, and that a panic escaping an ordinary `extern "C"` boundary leads to process abort. The panic hook runs before unwinding and is the appropriate place to capture useful panic-location information. [Rust Documentation](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html)

My recovery policy would be:

**Record diagnostics → mark the kernel faulted → stop further authoritative advancement → preserve the last good presentation → restore from a known-good checkpoint or restart.**

Catching a panic does not roll back partially modified world state. `AssertUnwindSafe` is an assertion about invariants, not a repair mechanism.

Do not represent this as crash isolation. `catch_unwind` is not protection against access violations, undefined behavior, process aborts, or every allocation failure. Nor should C++ exceptions be allowed to unwind through the bridge; Rust documents important restrictions around foreign unwinding. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

Keep panic-reporting paths simple and bounded. Avoid callbacks into Unreal from a panic hook. With Rayon, also account for its documented default abort behavior for panics in jobs without an ordinary propagation destination. [Docs.rs](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html)

### 4.4 Configure crash reporting deliberately

A caught panic is not necessarily a native crash, so report it through your own diagnostic/error path.

For genuine crashes, Epic documents that **CrashReportClient is not packaged by default**, and packaged applications need their own reporting configuration. Add kernel build ID, world identifier, simulation tick, save schema, and recent command identifiers to crash context **before** a failure occurs. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/crash-reporting-in-unreal-engine)

Do not assume Rust-created threads automatically receive exactly the same crash-handling coverage as every Unreal-created thread. Make symbolicated crashes from both the game thread and a Rust worker explicit Shipping-build tests.

Avoid installing competing process-level crash handlers from multiple libraries without deciding which one owns reporting.

---

## 5. DLL replacement during editor sessions

### 5.1 Start with a conservative lifecycle

For the first implementation:

**Load once per editor process; create/destroy a kernel per PIE session; restart the editor to change Rust code.**

Then add a dedicated **Reload Rust Kernel** action. Do not make arbitrary rebuild completion immediately unload an active kernel.

In packaged games, retain the module until application shutdown. TCE gains little from supporting production DLL replacement, while it inherits considerable additional lifecycle risk.

### 5.2 Use unique shadow copies

Compile into a staging location, then publish a completed artifact under a unique editor filename, such as:

`Saved/TCE/KernelCopies/<process>/<generation>/tce_kernel_<generation>.dll`

Load that copy, leaving the build output available for the next compilation. Publish its matching symbols and manifest together. Use a unique basename as well as a directory, and audit dependent DLL resolution separately: loading a uniquely named top-level DLL does not automatically version every dependency. Windows documents the distinction between locating a module and locating its dependencies. [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw)

### 5.3 Safe replacement is a state machine

I would implement the transition in this order:

1. **Reject new submissions** and stop initiating callbacks or asynchronous requests into the old kernel.
2. **Quiesce at a defined simulation boundary**, optionally producing a versioned checkpoint.
3. **Drain outstanding work and join every owned thread**, including worker pools and background persistence work.
4. **Release all old-kernel objects and borrows** while its code is still loaded.
5. **Invalidate handles and function pointers**, then unload only when all remaining lifetime obligations are satisfied.
6. **Load and validate the replacement**, create a new instance, restore through the supported save schema, and publish a new epoch.

Microsoft warns against complex work and thread synchronization from `DllMain`, where loader-lock constraints apply. Perform coordinated shutdown through explicit API calls before unloading; `FreeLibrary` can remove the module’s code when its reference count reaches zero. [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-best-practices)

Never pass the old Rust heap graph to a newly compiled DLL as “preserved state.” A compatible C ABI does not imply compatibility of internal Rust layout, allocation metadata, or destructors.

**On shutdown timeout, do not unload anyway or forcibly kill the thread.** Keep the module mapped and require a process restart. Microsoft documents that forced thread termination prevents normal cleanup. [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/procthread/terminating-a-thread)

### 5.4 Audit thread-local and global registrations

Even after joining your workers, investigate TLS initialized on long-lived Unreal threads, callbacks registered with other libraries, logging hooks, and other global registrations.

Rust documents that TLS values have thread-exit destruction behavior and platform-specific caveats. This makes TLS another lifetime obligation—not something automatically discharged by destroying a world handle. [Rust Documentation](https://doc.rust-lang.org/std/thread/struct.LocalKey.html)

For a solo developer, a useful intermediate strategy is to **stop and destroy the old kernel but retain its DLL mapping until editor exit**. That trades some editor memory for a safer iteration path. Bound the number of retained generations and restart periodically.

### 5.5 Live Coding is separate

Unreal Live Coding rebuilds and patches C++ binaries; it does not run Cargo or manage a Rust DLL’s objects. Epic also documents pointer invalidation and destructor hazards associated with object reinstancing. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-live-coding-to-recompile-unreal-engine-applications-at-runtime)

Therefore, keep the two workflows explicit:

**Rust implementation change:** controlled kernel replacement.

**C++ bridge change:** quiesce the kernel before patching lifecycle/callback code; restart for ABI or reflected-layout changes.

Do not let AI-generated edits quietly change both sides of the ABI while either side remains active.

---

## 6. Performance expectations and measurement

### What the evidence supports

A C ABI call is a native function call, not inherently a serialization operation. However, this does not justify claiming that the bridge is “free,” or that Rust will outperform equivalent C++. Data transformation, allocations, synchronization, cache behavior, and presentation updates all remain real costs. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

I found no defensible published number for **UE 5.8 + this DLL architecture + 50,000 TCE-like agents**. Existing integration projects establish feasibility, not the requested workload’s frame rate.

### A useful bandwidth calculation

Assume a compact presentation record of **64 bytes per person**:

| Quantity | Derived value |
| --- | --- |
| 50,000-person snapshot | 3.2 MB |
| Publication at 20 Hz | 64 MB/s of logical snapshot payload |
| Publication at 60 Hz | 192 MB/s |
| Three such Rust buffers | 9.6 MB |

These are arithmetic estimates, not measurements. Memory reads and writes, extra copies, cache effects, metadata, and GPU uploads add further traffic.

They nevertheless support a sensible starting decision: **measure one bulk copy before accepting the complexity of cross-thread, cross-language zero-copy ownership**.

### Proposed performance gates

At 60 fps, the frame interval is approximately **16.67 ms**. I would initially budget **under 1 ms p99 of game-thread time for bridge polling, copying, and dispatch**, excluding rendering and animation. This is a proposed budget, not a predicted result.

Benchmark the kernel headlessly, then inside UE with presentation disabled, then with realistic snapshots and rendering. Record simulation throughput alongside game-thread/render-thread frame percentiles, snapshot age, queue depth, allocation rate, and shutdown duration.

Most importantly, **50,000 simulated people should not imply 50,000 ticking Blueprint Actors**. Use presentation LOD, instancing, and selective high-detail characters. UE 5.8’s crowd work demonstrates transitions between individual Actors and instanced skinned representations, but its new MetaHuman Collections feature is explicitly Experimental and is not evidence that TCE’s full workload will meet 60 fps. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

For the RTX 4070 Ti, I would initially reserve GPU complexity for presentation rather than add GPU simulation and another synchronization boundary.

---

## 7. Precedents and what they actually establish

| Project or research | Verified scope/version | Lesson for TCE |
| --- | --- | --- |
| **[MaikKlein/unreal-rust](https://github.com/MaikKlein/unreal-rust?utm_source=chatgpt.com)** | README identifies UE 5.0 support and explicitly labels the project a proof of concept, not production-ready. | Useful examples of DLL integration, ECS, panic handling, and iteration. Do not assume UE 5.8 compatibility or adopt its entire engine-binding surface. [GitHub](https://github.com/MaikKlein/unreal-rust) |
| **[Author’s unreal-rust devlog](https://maikklein.github.io/unreal-rust-1/?utm_source=chatgpt.com)** | September 2022; discusses the architecture and development experience. | Particularly relevant discussion of Rust gameplay iteration and the difficulty of retaining state across code changes. Its broad panic-safety claims must be interpreted within Rust’s actual unwind limits. [Maik Klein](https://maikklein.github.io/unreal-rust-1/) |
| **[Uika](https://github.com/VioletHelianthus/uika)** | Describes UE 5.7+ support, a Rust DLL, a small C++ plugin, and function-pointer tables. The inspected README contains both a production-readiness warning and a notice that further maintenance is unlikely. | Direct precedent for independent Rust iteration through a function table. Also a clear illustration of the maintenance risk of comprehensive bindings. |
| **[thirdweb Unreal SDK](https://github.com/thirdweb-dev/unreal-engine)** | Distributed UE plugin with Rust core and documented Windows packaging. Its build rules link `libthirdweb.lib` and enumerate Windows system libraries. | Evidence that Rust-backed native functionality can be packaged as a UE product. Not evidence of an independently reloadable TCE-style kernel or 50k-agent throughput. |
| **[Firefox/Gecko Rust–C++ interoperability](https://firefox-source-docs.mozilla.org/writing-rust-code/cpp-interop.html?utm_source=chatgpt.com)** | Official production-engine interoperability guidance; not Unreal-specific. | Strong adjacent precedent for generated declarations and carefully designed Rust/C++ boundaries. It does not validate Unreal threading or DLL reload behavior. [Firefox Source Docs](https://firefox-source-docs.mozilla.org/writing-rust-code/cpp-interop.html) |
| **[RustBelt: paper and recorded talk](https://plv.mpi-sws.org/rustbelt/popl18/?utm_source=chatgpt.com)** | POPL 2018; foundational research into Rust and unsafe-library verification. | Supports the importance of explicit safety obligations around unsafe abstractions. It is not a UE integration paper or performance benchmark. [MPI SWS](https://plv.mpi-sws.org/rustbelt/popl18/) |

**Evidence gap:** I did not establish a publicly documented shipped game using this exact UE 5.8, independently threaded Rust-DLL simulation architecture. The strongest direct evidence is Epic’s supported native-library mechanism plus open-source Rust integration implementations. That is sufficient to justify the approach, but not to inherit someone else’s production validation.

---

## 8. Recommended implementation order and acceptance criteria

For TCE, I would implement the bridge in this order:

| Stage | Required result |
| --- | --- |
| **Native deployment** | An empty Rust kernel loads, negotiates ABI, and shuts down in a clean-machine Shipping package. |
| **Owned execution** | A coordinator and bounded workers start/stop repeatedly without detached threads or leaked world instances. |
| **Data exchange** | Batched commands and complete snapshots work under queue saturation and deliberately stalled rendering. |
| **Failure handling** | Exported-call and worker panics fault the kernel; native crash tests produce useful symbolicated reports. |
| **Lifecycle stress** | Repeated PIE, multiple world instances, save/load, and shutdown during work behave correctly. |
| **Editor replacement** | Shadow-copy replacement works before attempting state-preserving reload; full unloading requires the additional lifetime audit. |

Maintain one short ABI contract document that records ownership, allowed calling threads, blocking behavior, error semantics, and teardown order. Require every change to `unsafe` FFI or lifecycle code—including AI-generated changes—to update the relevant tests.

**The best fit for TCE is not the most elaborate Rust–Unreal integration. It is the smallest integration that makes authority, ownership, and shutdown explicit.** Keep the simulation independent, keep rendering asynchronous, batch the boundary, ship a conventional native DLL, and spend complexity on simulation behavior rather than a second gameplay-binding framework.

---

## Source map and version applicability

The project links above contain code and development discussion. These are the principal implementation references:

| Area | Documentation | Applicability |
| --- | --- | --- |
| UE library loading and staging | [Third-party libraries](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine?utm_source=chatgpt.com) | Retrieved as UE 5.8 documentation. |
| UBT integration | [Module properties](https://dev.epicgames.com/documentation/en-us/unreal-engine/module-properties-in-unreal-engine?utm_source=chatgpt.com), [target rules](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-engine-build-tool-target-reference?utm_source=chatgpt.com) | UE 5.8; verify project-specific compiler/SDK selection. |
| Rendering and editor iteration | [Threaded rendering](https://dev.epicgames.com/documentation/en-us/unreal-engine/threaded-rendering-in-unreal-engine?utm_source=chatgpt.com), [Live Coding](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-live-coding-to-recompile-unreal-engine-applications-at-runtime?utm_source=chatgpt.com) | UE 5.8 documentation; some rendering examples retain older macro styles. |
| Rust target and ABI | [Windows MSVC target](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html?utm_source=chatgpt.com), [type layout](https://doc.rust-lang.org/reference/type-layout.html?utm_source=chatgpt.com), [FFI guide](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | Current documentation retrieved September 2026. |
| Rust build configuration | [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html?utm_source=chatgpt.com), [code-generation options](https://doc.rust-lang.org/rustc/codegen-options/index.html?utm_source=chatgpt.com) | Pin actual build tools rather than following rolling documentation implicitly. |
| Panic and thread termination | [`catch_unwind`](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html?utm_source=chatgpt.com), [`JoinHandle`](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html?utm_source=chatgpt.com), [`thread::scope`](https://doc.rust-lang.org/std/thread/fn.scope.html?utm_source=chatgpt.com) | Retrieved std documentation: Rust 1.98.1. |
| Worker pools and buffering | [Rayon `ThreadPoolBuilder`](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html?utm_source=chatgpt.com), [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/?utm_source=chatgpt.com) | Retrieved versions: Rayon 1.12.0 and `triple_buffer` 9.0.0. Pin selected releases. |
| Windows lifetime and CRT | [DLL best practices](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-best-practices?utm_source=chatgpt.com), [CRT objects across DLL boundaries](https://learn.microsoft.com/en-us/cpp/c-runtime-library/potential-errors-passing-crt-objects-across-dll-boundaries?view=msvc-170&utm_source=chatgpt.com) | Windows native-runtime contracts; applicable to this Win64 design. |
| Crash deployment | [Unreal crash reporting](https://dev.epicgames.com/documentation/en-us/unreal-engine/crash-reporting-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8; packaged reporting requires explicit configuration. |

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9279b-eaf4-83ea-9356-e1336df45d26)
