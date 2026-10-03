# Embedding HTML/TypeScript UI in Unreal Engine 5.8

**Engineering report for The Civilization Engine — research checked September 27, 2026**

## Executive recommendation

**For TCE, the strongest production choice is Coherent Gameface, connected through a small, asynchronous C++ interface to the Rust simulation DLL.** Keep the frontend independent of that interface through a TypeScript transport adapter. Use a local WebSocket as an optional development and external-inspector transport, rather than making it a mandatory part of the packaged game.

That recommendation is conditional on two gates: **an affordable license and a successful packaged UE 5.8/Windows/D3D12 test with TCE’s actual tables, charts, fonts, and input behavior.** Gameface provides documented Unreal integration, input handling, profiling, packaging, and JavaScript binding, with a particularly relevant shipped precedent in *Civilization VII*. Those are stronger reasons to choose it than unverified performance claims. [Coherent Labs](https://docs.coherent-labs.com/unreal-gameface/)

**The budget fallback is Unreal’s built-in CEF browser, wrapped in a thin C++ host rather than used only as a Blueprint WebBrowser widget.** Also include **UCefView 1.1.0** in the initial comparison when shared-texture rendering matters: its current documentation explicitly lists UE 5.8 and D3D12 shared-texture support. Do not begin by maintaining a custom Chromium fork or building an Ultralight-to-Unreal renderer yourself. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/WebBrowser/SWebBrowser)

The architectural principle matters more than the renderer:

> **The UI should query and subscribe to compact simulation projections—not mirror 50,000 people into JavaScript.**

### Version and evidence caveats

Epic’s referenced browser APIs are documented for **UE 5.8**, but I did **not** verify the exact Chromium/CEF build bundled with your particular 5.8 installation. Record that from the actual engine build rather than borrowing a number from an older tutorial. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/WebBrowser/SWebBrowser/BindUObject)

Gameface’s public documentation currently identifies itself as **3.1.2.1**. Its requirements table stops at UE 5.7, while its known-issues page explicitly discusses **UE 5.8.0**. This is inconsistent documentation—not sufficient evidence that 5.8 is unsupported. Confirm the exact supported SDK/plugin package. The documented 5.8 issue concerns **Vulkan Dynamic Rendering**, not TCE’s proposed Windows D3D12 path. [Coherent Labs](https://docs.coherent-labs.com/unreal-gameface/information/requirements/)

No performance measurements below were run on your hardware. Calculations, proposed budgets, vendor claims, and shipped-project evidence are identified separately.

---

## 1. Options: how they work and where they fit

### Comparison at a glance

The suitability judgments in this table are engineering assessments for TCE, not benchmark rankings.

| Option | Main advantage | Main liability | Fit for TCE |
| --- | --- | --- | --- |
| **UE WebBrowser / CEF** | Existing Unreal integration and Chromium web compatibility | Rendering path, focus behavior, runtime footprint, and engine-bound browser updates need verification | Best baseline and budget fallback |
| **Coherent Gameface** | Game-oriented rendering, binding, input, profiling, and commercial support | Quoted license; not identical to a current desktop browser | Preferred production option after acceptance testing |
| **UCefView** | Explicit UE 5.8 support and D3D12 shared-texture path | Additional plugin/runtime dependency; support maturity needs evaluation | Strong candidate when retaining Chromium is important |
| **Ultralight** | Embeddable HTML renderer with a custom GPU interface | More Unreal-specific integration responsibility | Attractive technology, but not the lowest-risk solo choice |
| **BLUI** | Publicly inspectable CEF/Unreal implementation | You must establish current engine compatibility and maintenance expectations | Useful code precedent; secondary candidate |
| **WebView2** | Microsoft-maintained web runtime and Windows integration | Not a ready-made Unreal texture/UMG integration | Better for companion tools than TCE’s main overlay |
| **RmlUi** | Lightweight, controllable native HTML/CSS-like UI | Not a drop-in HTML/TypeScript browser runtime | Only if willing to change the frontend requirement |

These distinctions follow the respective integration documentation and code repositories. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/WebBrowser/SWebBrowser)

### 1.1 Unreal WebBrowser widget: CEF through Slate/UMG

On Windows, the relevant conceptual stack is:

```
HTML/CSS/JavaScript
        ↓
CEF browser and renderer processes
        ↓
Unreal WebBrowser window / texture interface
        ↓
SWebBrowser
        ↓
UMG or custom Slate host
```

This is an embedded, offscreen-rendered browser displayed by Unreal, rather than simply an operating-system browser window placed over the game. Epic exposes texture, viewport, input, navigation, and lifecycle interfaces through its browser APIs. [Chromium Embedded](https://chromiumembedded.github.io/cef/general_usage)

The stock `UWebBrowser` exposes convenient operations such as loading a URL or HTML string and executing JavaScript. For TCE, a small C++ wrapper around `SWebBrowser` gives better control over binding, navigation policy, browser frame rate, input-method integration, and shutdown. These controls are documented in the lower-level API. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/WebBrowserWidget/UWebBrowser)

**Transparency.** Unreal exposes `SupportsTransparency`; the page must also leave its root background transparent. However, **visual transparency is not a promise of click-through behavior**. Slate input routing and DOM input routing are separate concerns. Do not assume that CSS `pointer-events: none` will make the underlying game viewport receive an event that the native browser widget already captured. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/WebBrowser/SWebBrowser/FArguments)

For TCE, prefer bounded rectangular panels where practical. For a fullscreen transparent UI, maintain explicit interactive regions in the native host, or use the middleware’s supported input-forwarding mechanism. Avoid synchronous JavaScript hit tests or GPU pixel readbacks for each pointer event.

**Input focus.** The browser API includes input-method and focus-related integration, but that does not replace application-level ownership rules. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/WebBrowser/SWebBrowser) TCE should explicitly define:

* When a text field owns keyboard input, camera shortcuts must stop.
* Escape should unwind a modal or return focus to the world predictably.
* Scrolling, dragging, alt-tab, DPI changes, and input-method composition must work in the packaged application.

Treat Chinese/Japanese/Korean input, candidate-window placement, clipboard operations, and gamepad navigation as acceptance tests—not polish to defer until release.

**Performance.** Avoid two opposite myths: “CEF always uploads a CPU bitmap” and “CEF is automatically zero-copy.” Current upstream CEF supports both `OnPaint`, which supplies BGRA pixel buffers, and `OnAcceleratedPaint`, which exposes shared GPU resources. Upstream capability does not establish which path a specific Unreal integration actually uses. The current accelerated API also places strict limits on shared-resource lifetime and instructs clients to copy into their own texture. [GitHub](https://raw.githubusercontent.com/chromiumembedded/cef/master/include/cef_render_handler.h)

**Package size.** CEF brings a browser runtime, subprocess executable, and supporting resources—not merely the Unreal widget code. The required distribution set is version-dependent. I found no trustworthy universal “UE 5.8 WebBrowser adds exactly X MB” figure; measure the staged package difference for your build. [Chromium Embedded](https://chromiumembedded.github.io/cef/general_usage)

**Assessment:** entirely credible for TCE’s panels, provided the packaged test passes. It should not be rejected merely because it is the built-in option, nor assumed production-complete because a page displays correctly in the editor.

### 1.2 Coherent Gameface

Gameface is game UI middleware using HTML, CSS, and JavaScript, with its own game-oriented integration rather than a generic Chromium embedding. Its Unreal documentation covers UMG integration, JavaScript interactions, input, packaging, render features, and profiling. [Coherent Labs](https://docs.coherent-labs.com/unreal-gameface/)

Its principal advantage for TCE is reducing the amount of engine-integration behavior you must own. The profiler integration exposes CPU/GPU and memory information relevant to diagnosing UI work alongside Unreal rendering. [Coherent Labs](https://docs.coherent-labs.com/unreal-gameface/performance-optimization/profilingoverview/)

The main technical caveat is **web compatibility**. Do not choose a complex charting or component library solely because it works in desktop Chrome. Coherent’s June 2024 chart tutorial, for example, documented full support for D3 v4 and earlier but only partial support for newer versions. That is historical, version-specific evidence—not a claim about every current library—but it demonstrates why an actual compatibility test is necessary. [Coherent Labs](https://coherent-labs.com/blog/uitutorials/charts/)

For a new frontend, Coherent now publishes a **SolidJS, TypeScript, and Vite** project template and component collection. That is a useful starting point for AI coding agents because it supplies concrete, inspectable conventions rather than requiring them to infer compatible patterns. It is not a reason to rewrite an already-working React UI automatically. [GitHub](https://github.com/CoherentLabs/Gameface-UI)

**Input pitfall:** Gameface documents different integration paths for HUD input forwarding and UMG. It explicitly warns against combining `SCohtmlInputForward` with UMG-based views. Choose one route and make its ownership rules explicit. [Coherent Labs](https://docs.coherent-labs.com/unreal-gameface/advanced-features/input/)

**Commercial terms:** the public model is quoted per title and platform, with a license valid for the title’s lifetime and the first year of support/maintenance included. Source access is an additional paid option. Obtain the actual solo-project quote and clarify subsequent SDK-update access; do not interpret “lifetime license” as unlimited lifetime maintenance. [Coherent Labs](https://coherent-labs.com/pricing/)

**Assessment:** my preferred production integration, but not a purchase to make before testing TCE’s hardest UI screen.

### 1.3 UCefView and BLUI: alternative CEF integrations

**UCefView 1.1.0** currently documents UE 5.0–5.8 support, Slate and UMG components, and shared-texture transfer for Windows D3D11/D3D12. CEF runs in a separate `XCefHost` process, allowing coexistence with Unreal’s browser. Its packaged games include the target runtime; the shared SDK installation is a development workflow, not a player prerequisite. These are documented capabilities, not independently measured performance results. [CEFView](https://cefview.github.io/UCefView/)

That makes it worth testing before attempting your own accelerated CEF integration. I would still require a packaged stress test, maintenance history review, license review, and a clear update/support policy.

**BLUI** provides inspectable Unreal/CEF code and a JavaScript event bridge. Its current repository identifies CEF **128.0.6613.138**—a BLUI dependency, **not** evidence of Unreal’s bundled version. The repository also documents renaming its CEF DLLs from version 5.1.0 to prevent collisions with Unreal’s copies. This is a useful warning about loading multiple browser distributions into an application. [GitHub](https://github.com/getnamo/BLUI-Unreal)

**Assessment:** UCefView is a serious current candidate; BLUI is especially useful as an implementation precedent. Neither should be treated as a proven UE 5.8 shipping solution merely because a repository or demo exists.

### 1.4 Ultralight

Ultralight combines web-engine components with its own rendering architecture. It offers a straightforward surface-based integration path and a custom `GPUDriver` interface for native GPU rendering. The latter has D3D11 and D3D12 reference implementations, but those are not the same thing as a maintained UE 5.8 RHI integration. [Ultralight](https://docs.ultralig.ht/docs/architecture)

The attraction is control over rendering and embedding. The cost is owning more of resource loading, fonts, input, graphics-resource lifetime, and Unreal-specific integration.

Its current pricing page lists a free tier for qualifying indie developers below the stated $100,000 threshold, with limited features and performance. Pro is **US$3,000 per year per application**, subject to its eligibility and license terms. Therefore, “Ultralight is free” is not a sufficient budgeting assumption. [Ultralight](https://ultralig.ht/pricing)

**Assessment:** reasonable when someone already owns the graphics integration or a suitable supported plugin exists. For a solo developer building the simulation itself, it is an unnecessary integration project unless the alternatives fail.

### 1.5 WebView2, RmlUi, and native alternatives

**WebView2** offers Evergreen and fixed-version runtime deployment. Evergreen updates are managed separately; fixed-version deployment gives explicit version control but adds a runtime that Microsoft documents as **over 250 MB**. Its composition controller supports DirectComposition integration, but that is not a drop-in Unreal texture widget. Input, composition, fullscreen behavior, capture, and lifecycle still require a host implementation. [Microsoft Learn](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)

I would use it more readily for a separate TCE launcher or inspector than for the central in-game UI.

**RmlUi** is a C++ HTML/CSS-like UI library with an embedding-oriented renderer interface. Its document model and scripting approach are not a drop-in replacement for a browser running an existing TypeScript application. It becomes attractive only if TCE relaxes that requirement. [GitHub](https://github.com/mikke89/RmlUi)

Regardless of the web renderer, I recommend keeping a minimal **native Unreal emergency menu** for pause, quit, and UI-reload diagnostics. A failed page should not remove the player’s ability to exit or recover.

---

## 2. Trade-offs and performance evidence

### 2.1 What the available benchmarks establish

I did not find a controlled, directly comparable benchmark of stock UE 5.8 CEF, Gameface, Ultralight, and UCefView running the same data-heavy scene on an RTX 4070 Ti.

Coherent advertises UI performance below one millisecond, and Ultralight advertises a substantially lighter footprint than Chromium. Those are vendor claims without a directly transferable TCE workload and measurement methodology. They should inform what to investigate, not become entries in TCE’s frame budget. [Coherent Labs](https://coherent-labs.com/products/coherent-gameface/)

The strongest available evidence is instead:

| Evidence | What it supports | What it does not support |
| --- | --- | --- |
| Documented GPU/shared-texture interfaces | A potentially efficient transfer path exists | The complete UI will fit within a particular millisecond budget |
| Shipped Gameface strategy-game integration | The technology can support production strategy UI | Your libraries and UE configuration will work unchanged |
| Public CEF integrations and samples | The implementation is inspectable and reproducible | Current shipping quality on your exact target |
| Vendor runtime-size documentation | A deployment characteristic of that product | A comparable compressed package size for every alternative |

The underlying interfaces and shipped case are documented by their implementers. [GitHub](https://raw.githubusercontent.com/chromiumembedded/cef/master/include/cef_render_handler.h)

### 2.2 Quantitative implications at 1440p

These are **calculations**, not benchmark results.

A 2560 × 1440 BGRA8 surface contains:

\[
2560 \times 1440 \times 4
= 14{,}745{,}600\text{ bytes}
\approx 14.06\text{ MiB}.
\]

| Scenario | Calculated amount |
| --- | --- |
| One full-resolution BGRA8 surface | 14.06 MiB |
| Three such surfaces | 42.19 MiB |
| One full-surface transfer, 30 times/second | 0.442 GB/s |
| One full-surface transfer, 60 times/second | 0.885 GB/s |

These numbers exclude browser heaps, decoded images, font caches, additional render targets, synchronization, and further copies. Dirty-region updates and shared-texture paths can change the actual transfer workload; CEF exposes both concepts. [GitHub](https://raw.githubusercontent.com/chromiumembedded/cef/master/include/cef_render_handler.h)

The implication is **not** that a fullscreen browser must be slow. It is that view size, repaint frequency, and the selected transfer path belong in the measurement plan.

The data side can be worse. Suppose a naïve UI snapshot averages only **200 bytes per person**:

\[
50{,}000 \times 200 \times 10\text{ updates/s}
=100\text{ MB/s}.
\]

That is before additional serialization, allocation, parsing, and component-update work. No choice between a native bridge and WebSocket repairs this architecture.

### 2.3 Separate the three clocks

For TCE, I recommend treating these independently:

| Clock | Proposed starting policy |
| --- | --- |
| Game presentation | 60 fps |
| UI animation and interaction | Up to 60 fps while active |
| Simulation-to-panel data refresh | Usually 5–10 Hz; slower for historical aggregates |

These are design starting points, not middleware limits.

A population panel can scroll smoothly while population totals refresh five times per second. A century-scale production chart usually needs downsampled historical points for its current viewport—not every underlying event. Hidden panels should unsubscribe or substantially reduce work.

### 2.4 Proposed acceptance budgets

For the specified PC, start with these **engineering targets**, then revise from measurements:

| Measurement | Initial target |
| --- | --- |
| Total frame-time requirement | 16.67 ms for 60 fps |
| UI-attributable game-thread work | Approximately ≤1 ms at p95 |
| UI-attributable GPU work | Approximately ≤1 ms at p95 |
| Closed-panel data subscriptions | None unless explicitly required |
| Queued UI state updates | Bounded; obsolete state replaced rather than accumulated |
| Long-session memory | No unbounded growth across repeated panel/world reloads |

CPU and GPU work overlap, so do not simply add unrelated profiler counters and call the result frame time. Compare the full application against a no-web-UI baseline, and inspect p95/p99 latency and individual spikes—not average FPS alone.

The 64 GB system-memory capacity gives room, but does not protect the game thread from expensive JSON handling, layout, or synchronization.

---

## 3. Precedents and lessons

### Civilization VII: the closest product precedent

In a June 2025 vendor-hosted interview, Firaxis describes using Gameface in *Civilization VII*, discussing integration with its own engine and the value of threading flexibility and Chrome-style debugging/profiling tools. This is relevant evidence for a data-heavy strategy interface, although **it is not an Unreal integration case or a quantitative performance benchmark**. [Coherent Labs](https://coherent-labs.com/blog/casestudies/firaxis-case-study/)

**Lesson for TCE:** invest in profiling, inspectable frontend code, and engine/UI separation. The existence of shipped strategy UI is more meaningful than a generic animated-menu demonstration.

### FiveM/RedM NUI: CEF overlays with explicit focus and messaging

Cfx.re documents fullscreen CEF-based NUI, JSON messaging, callback mechanisms, resource packaging, and a focus stack. Its documentation explicitly notes that fullscreen resources do not provide click-through between one another. It also documents a move from the older `nui://` resource scheme to secure-context-compatible resource origins. [Cfx Documentation](https://docs.fivem.net/docs/scripting-manual/nui-development/full-screen-nui/)

**Lesson for TCE:** focus policy, resource origins, and callback completion are first-class parts of the integration. Chromium upgrades can affect loading and security behavior even when your TypeScript has not changed.

### BeamNG.drive: subscription-driven simulation UI

BeamNG’s UI-app documentation shows HTML/JavaScript apps using named data streams, including registering streams and removing them when an app is destroyed. The referenced documentation includes a legacy AngularJS app API; it should not be read as a complete description of BeamNG’s entire current frontend. [BeamNG Documentation](https://documentation.beamng.com/modding/ui/app_creation/)

**Lesson for TCE:** panels should declare their data requirements and clean them up. This is a better model than broadcasting all simulation data to every screen.

### Open-source integration lessons

BLUI’s DLL-renaming change demonstrates that native dependency collisions are a real integration concern. UCefView’s separate-host design represents a different approach to coexistence. These are useful architectural precedents, not interchangeable proofs of shipping maturity. [GitHub](https://github.com/getnamo/BLUI-Unreal)

### Foundational paper: browser separation is not automatic application security

Barth, Jackson, Reis, and the Chrome team’s **2008 technical report, “The Security Architecture of the Chromium Browser,”** explains the motivation for separating privileged browser functions from rendering processes. It is foundational architecture reading—not evidence that a particular CEF embedding enables every sandbox protection, and not a modern performance comparison. [Stanford Computer Security Laboratory](https://seclab.stanford.edu/websec/chromium/chromium-security-architecture.pdf)

**Lesson for TCE:** the native interface remains a privileged boundary. Browser process separation does not make an unrestricted `execute_native_command()` endpoint safe.

---

## 4. Recommended TCE implementation

### 4.1 Architecture

I recommend this arrangement:

```
Rust simulation DLL
  authoritative world state
  command queue
  UI query/projection service
            │
       versioned C ABI
            │
Unreal C++ UI subsystem
  lifetime and thread ownership
  validation and batching
  native middleware adapter
            │
   asynchronous UI protocol
            │
TypeScript transport interface
            │
Panels, tables, charts, search
```

Add a parallel development path:

```
Desktop browser + development server
            │
  authenticated loopback WebSocket
            │
Same query/projection service
```

The renderer and transport are **independent choices**. A WebSocket does not remove the need to choose and maintain an embedded renderer.

### 4.2 Native bridge versus local WebSocket

| Concern | Native middleware bridge | Local WebSocket |
| --- | --- | --- |
| Packaged-game lifecycle | Fewer independently managed services | Requires server startup, connection, shutdown, and reconnection |
| Browser-based development | Requires an adapter or mock | Natural fit |
| External inspector | Requires another exposed interface | Natural fit |
| Coupling | Middleware-specific at the boundary | Protocol-oriented |
| Security surface | Privileged native methods and navigation policy | Those concerns plus listener, authentication, origin policy |
| Performance | Must still serialize/batch appropriately | Must still serialize/batch appropriately |

**Choose the native bridge for the packaged game.** Use WebSocket for browser-based development, automation, and an optional external inspector.

This is an operational recommendation, not a claim that WebSocket latency is inherently unacceptable. A well-designed local WebSocket can work; it simply adds responsibilities that TCE does not need for every player.

#### CEF-specific behavior

Epic’s `BindUObject` exposes an object under `window.ue`. Object, function, and property names are lowercased. Calls communicate asynchronously with the rendering process, and return values are wrapped in JavaScript Future objects. Permanent bindings remain visible to subsequent pages loaded in that widget. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/WebBrowser/SWebBrowser/BindUObject)

Therefore, expose a **small dedicated bridge object**, not arbitrary gameplay objects. Restrict navigation, retain the bridge with appropriate Unreal lifetime management, and unregister it during teardown.

#### Gameface-specific behavior

Gameface distinguishes native binding readiness from scripting readiness. Register handlers at the documented binding stage, and send events only when scripting is ready. Its documentation also notes that exposed JavaScript objects involve copying and recommends limiting transferred data. [Coherent Labs](https://docs.coherent-labs.com/unreal-gameface/integration/ui-scripting/jsinteractions/)

In either implementation, add a TCE-level `ui_ready` handshake after the frontend has installed its handlers. A completed page load is not sufficient proof that the application is ready for a snapshot.

### 4.3 Make the protocol independent of the middleware

Use three semantic operations:

```
query(request)       → one response
command(request)     → accepted/rejected, then eventual result
subscribe(request)   → snapshot followed by updates
```

Every message should carry a protocol version and request/subscription identifier. State updates should also carry the world generation, simulation tick, and sequence or base version.

Recommended rules:

**Query the backend, then virtualize the frontend.** Filtering and sorting 50,000 people should normally happen in the Rust query service. Return the visible page and a total count. Render only the visible rows plus a small margin.

**Use durable identifiers.** Send 64-bit entity IDs as decimal strings in JSON. Reject references to deleted entities and messages from an earlier world generation.

**Bound every queue.** Coalesce replaceable state updates to the latest value. Do not silently discard commands. Reject overload explicitly or apply controlled backpressure.

**Resynchronize deliberately.** On a missed delta, page reload, world replacement, or protocol mismatch, request a fresh snapshot rather than trying to apply uncertain incremental state.

**Begin with batched JSON.** Introduce binary encoding only after profiling shows serialization is a meaningful bottleneck. The first optimization should be fewer records and fewer updates.

**Never build executable JavaScript from unescaped simulation text.** Use structured binding or a serialized payload passed to a fixed receiver. Agent names, laws, books, and mod content are data—not code.

These are proposed TCE protocol requirements, rather than claims that a middleware automatically provides them.

### 4.4 Keep Rust ownership and threading explicit

The Rust DLL should own simulation state. C++ should exchange opaque handles and well-defined buffers through a versioned C ABI, with explicit allocation/freeing rules. Avoid passing Rust-owned `Vec` or `String` layouts directly as an ABI contract. Asynchronous callbacks require clear lifetime management, and callbacks must not remain registered after their target is destroyed. Rust’s FFI guidance specifically addresses these ownership, callback, and unwinding issues. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

For TCE, I recommend:

* Simulation commands enter a queue and take effect at defined simulation boundaries.
* UI queries consume consistent projections or snapshots, rather than locking the live simulation while JavaScript waits.
* Native callbacks are marshalled to the appropriate Unreal thread; shutdown cancels work and joins workers before unloading the DLL.

A Rust DLL inside Unreal is **not process isolation**. Design failure handling accordingly.

### 4.5 Secure the optional WebSocket

A local listener still needs security controls. OWASP recommends explicit origin validation, authentication, message validation, size limits, and rate limiting for WebSocket services. [OWASP Cheat Sheet Series](https://cheatsheetseries.owasp.org/cheatsheets/WebSocket_Security_Cheat_Sheet.html)

For TCE’s development adapter, bind only to loopback, choose an ephemeral port, require a random per-session token, and allow only expected origins. Keep tokens out of logs. Validate message kinds and lengths, and close the listener with the owning process.

Do not treat CORS as WebSocket authentication. Do not expose arbitrary filesystem access, console commands, or native function calls. Any future LAN/remote inspector should be a separately reviewed feature, not an accidental consequence of listening on all interfaces.

### 4.6 Package assets through a deliberate resource strategy

**A file inside Unreal’s packaged storage is not automatically a normal operating-system file.**

Epic distinguishes:

| Packaging setting | Intended access |
| --- | --- |
| **Additional Non-Asset Directories to Copy** | Files accessed outside Unreal’s UFS, using ordinary filesystem access |
| **Additional Non-Asset Directories to Package** | Files accessed through Unreal’s UFS-aware mechanisms |

Choose based on the browser’s resource loader. A browser attempting ordinary file access cannot be assumed to read assets merely because they were placed in a pak or IoStore container. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Developer/DeveloperToolSettings/UProjectPackagingSettings)

For a minimal CEF implementation, a staged loose frontend directory is often the simplest starting design. For an Unreal-aware middleware resource loader, use its documented packaged-resource path. Test relative asset URLs, JavaScript modules, MIME types, fonts, and origin-sensitive APIs in the actual package.

Gameface’s packaging documentation calls out its runtime binaries and the need to package raw font files where required; a Unreal font asset is not automatically a substitute for the web font resource. Some examples retain older Unreal paths, so apply the mechanism rather than copying old directory names literally. [Coherent Labs](https://docs.coherent-labs.com/unreal-gameface/integration/packaging/)

The shipping build should contain compiled frontend assets—not a Vite development server, Node.js runtime, CDN dependency, or developer-machine path. Keep writable caches and diagnostics outside the installation directory.

Also stage and validate the Rust DLL, browser subprocess/runtime dependencies, and third-party notices. Test on a clean machine without Unreal Editor or development SDKs installed.

### 4.7 Input and rendering acceptance tests

Before building dozens of panels, implement one deliberately difficult screen: a population table, a century-scale chart, a search field with international text, a modal, and a transparent region over the world.

| Test group | Required cases |
| --- | --- |
| Focus | Text entry blocks camera controls; Escape restores the correct owner; alt-tab recovers |
| Pointer routing | Transparent area reaches world; panel receives clicks; wheel does not zoom the world accidentally |
| Text | IME composition, clipboard, selection, Unicode, long labels |
| Windowing | Resize, fullscreen transitions, 100/125/150% DPI, multiple monitor scales |
| Rendering | Alpha edges, sRGB correctness, supported HDR mode, popup clipping |
| Lifecycle | Panel reopen, page reload, world load, save/load, shutdown with requests in flight |
| Data pressure | 50,000-row logical dataset, filtering, sorting, fast-forward, burst updates |
| Deployment | Offline launch, non-ASCII install path, read-only installation directory, missing/corrupt resource handling |

Do not infer these results from a successful browser-only test.

### 4.8 Automation for a solo developer using AI agents

Use two layers of tests.

**Frontend tests** run against deterministic mock snapshots and protocol fixtures in a normal browser. They verify component behavior, formatting, virtualization, and query semantics. Playwright is useful here. Its CDP connection mode is Chromium-only and explicitly lower-fidelity than its normal protocol, so attaching it to an embedded runtime should be treated as an additional capability—not the sole test strategy. [Playwright](https://playwright.dev/docs/api/class-browsertype)

**Embedded tests** run in packaged Unreal and verify the native boundaries: focus, texture composition, input-method behavior, DLL loading, resource staging, and recovery. Gameface also documents an E2E framework using its debugging protocol and Mocha, which is worth evaluating for renderer-specific automation. [Coherent Labs](https://coherent-labs.com/blog/news/e2e-testing-framework/)

Have AI agents work primarily on typed schemas, frontend components, mock scenarios, and reproducible tests. Keep unsafe FFI, native lifetime management, and RHI/resource synchronization behind small interfaces that receive deliberate review.

### 4.9 Adoption sequence

**First, establish the vertical slice.** Build the difficult screen against mock data and package it in UE 5.8/D3D12. Evaluate Gameface and stock CEF; add UCefView when stock CEF’s measured transfer path or browser version is a problem.

**Second, decide from evidence.** Compare compatibility, focus correctness, p95/p99 frame behavior, process-tree memory, installed package size, and recovery behavior. Include the license and update policy in the decision.

**Third, connect Rust through the stable protocol.** Add subscriptions, query paging, command acknowledgements, and world-generation invalidation. Run repeated load/unload tests and a long-duration soak.

Then freeze an exact engine/plugin/runtime/frontend dependency set in a build manifest. Upgrade deliberately and rerun the packaged acceptance suite.

---

## 5. Linked sources and version applicability

The following are the most useful implementation references. Current documentation should be pinned or archived alongside the integration because default documentation pages can change.

| Area | Primary sources | Applicability and cautions |
| --- | --- | --- |
| Unreal browser host | [SWebBrowser](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/WebBrowser/SWebBrowser?utm_source=chatgpt.com), [construction arguments](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/WebBrowser/SWebBrowser/FArguments?utm_source=chatgpt.com), [BindUObject](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/WebBrowser/SWebBrowser/BindUObject?utm_source=chatgpt.com) | UE 5.8 API documentation; does not establish the exact bundled Chromium version |
| Unreal packaging | [UProjectPackagingSettings](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Developer/DeveloperToolSettings/UProjectPackagingSettings?utm_source=chatgpt.com) | UFS versus NonUFS staging distinction |
| CEF architecture and code | [General usage](https://chromiumembedded.github.io/cef/general_usage?utm_source=chatgpt.com), [render-handler source](https://github.com/chromiumembedded/cef/blob/master/include/cef_render_handler.h) | Upstream documentation/source; verify against the branch actually embedded |
| Gameface integration | [Unreal documentation](https://docs.coherent-labs.com/unreal-gameface/?utm_source=chatgpt.com), [input](https://docs.coherent-labs.com/unreal-gameface/advanced-features/input/?utm_source=chatgpt.com), [JavaScript interactions](https://docs.coherent-labs.com/unreal-gameface/integration/ui-scripting/jsinteractions/?utm_source=chatgpt.com), [packaging](https://docs.coherent-labs.com/unreal-gameface/integration/packaging/?utm_source=chatgpt.com) | Public docs identify version 3.1.2.1 |
| Gameface compatibility and licensing | [Requirements](https://docs.coherent-labs.com/unreal-gameface/information/requirements/?utm_source=chatgpt.com), [known issues](https://docs.coherent-labs.com/unreal-gameface/knownissues/?utm_source=chatgpt.com), [pricing](https://coherent-labs.com/pricing/?utm_source=chatgpt.com) | Requirements table and UE 5.8 issue notes are inconsistent; confirm exact SDK |
| Gameface code and testing | [Gameface UI source](https://github.com/CoherentLabs/Gameface-UI?utm_source=chatgpt.com), [E2E testing overview](https://coherent-labs.com/blog/news/e2e-testing-framework/?utm_source=chatgpt.com) | Inspectable frontend template; E2E article published May 2025 |
| Video walkthrough | [Setting up a React project in Gameface](https://www.youtube.com/watch?v=r7yLw2Vu2OM&utm_source=chatgpt.com) | Vendor instructional video; historical workflow guidance, not a current API guarantee |
| Ultralight | [Game integration](https://docs.ultralig.ht/docs/integrating-with-games?utm_source=chatgpt.com), [custom GPUDriver](https://docs.ultralig.ht/docs/using-a-custom-gpudriver?utm_source=chatgpt.com), [source repository](https://github.com/ultralight-ux/Ultralight?utm_source=chatgpt.com), [pricing](https://ultralig.ht/pricing?utm_source=chatgpt.com) | Confirm SDK version and license tier; reference GPU drivers are not a UE integration |
| Alternative CEF integrations | [UCefView documentation](https://cefview.github.io/UCefView/?utm_source=chatgpt.com), [UCefView repository](https://github.com/CefView/UCefView), [BLUI source](https://github.com/getnamo/BLUI-Unreal?utm_source=chatgpt.com) | UCefView docs: 1.1.0; BLUI’s CEF version is independent of Epic’s |
| Other renderers | [WebView2 distribution](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution?utm_source=chatgpt.com), [composition controller](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2compositioncontroller), [RmlUi source](https://github.com/mikke89/RmlUi?utm_source=chatgpt.com) | Windows-runtime alternative versus HTML/CSS-like native UI |
| Shipped precedents | [Firaxis interview](https://coherent-labs.com/blog/casestudies/firaxis-case-study/?utm_source=chatgpt.com), [FiveM fullscreen NUI](https://docs.fivem.net/docs/scripting-manual/nui-development/full-screen-nui/?utm_source=chatgpt.com), [BeamNG UI apps](https://documentation.beamng.com/modding/ui/app_creation/?utm_source=chatgpt.com) | Distinguish shipped practice from UE-specific support and measured performance |
| Paper and boundary safety | [Chromium security architecture, 2008](https://seclab.stanford.edu/websec/chromium/chromium-security-architecture.pdf?utm_source=chatgpt.com), [Rust FFI guidance](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com), [OWASP WebSocket security](https://cheatsheetseries.owasp.org/cheatsheets/WebSocket_Security_Cheat_Sheet.html?utm_source=chatgpt.com) | Foundational architecture and implementation safeguards, not renderer benchmarks |

## Final decision

**Choose Gameface when its quote and exact UE 5.8 package pass the vertical slice. Otherwise, ship a carefully wrapped CEF integration; evaluate UCefView before taking ownership of browser-renderer internals.**

For TCE, the largest avoidable risk is not choosing the “wrong” HTML renderer. It is building a frontend that receives the whole simulation, depends on implicit focus behavior, and is only tested inside the editor.

Keep the simulation authoritative, the interface asynchronous, the data bounded, and the packaged-game acceptance tests mandatory.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92981-314c-83ea-8b6c-896d0852cede)
