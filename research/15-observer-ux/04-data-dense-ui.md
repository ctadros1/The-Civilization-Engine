# Designing data-dense observer panels for TCE

*Factorio’s production interface is a useful reference: trends, exact values, filters, and time ranges coexist in one analytical workspace. The important lesson is the relationship between those elements, not its particular visual style.* [Factorio](https://factorio.com/blog/post/fff-337)

## Recommendation

**Design TCE as a linked observation workspace, not as a continuously refreshed dump of simulation state.** The observer should be able to move smoothly from “something changed” to “who was affected,” then to “what recorded mechanisms contributed.”

For the initial web stack, I recommend **Apache ECharts for general analytical charts, TanStack Table with TanStack Virtual for ledgers, and a custom Canvas timeline using D3’s scales and interaction utilities**. Add **Sigma.js with Graphology** when relationship exploration becomes a first-class feature. Keep **uPlot** as a specialist option for high-frequency time-series panels rather than introducing multiple chart engines immediately.

The central engineering rule is:

> **Simulate every person; transmit the requested evidence; render only what the observer can currently inspect.**

The following separates published evidence from proposed TCE design choices. **None of the retrieved benchmarks establishes performance inside TCE’s packaged UE5 webview.** That requires a representative in-engine test.

---

## 1. How the panels should work

### Organize the interface around questions

A population ledger, chart, timeline, and network should be alternative views of the same selection—not separate applications with independent filters and clocks.

Research on interactive visual analysis treats filtering, selection, navigation, coordinated views, and recording an investigation as complementary activities. That is a better foundation for TCE than maximizing the number of simultaneously visible charts. [ACM Digital Library](https://dl.acm.org/doi/10.1145/2133806.2133821)

I would implement these observation paths:

| Observer’s question | Primary view | Supporting view | Required interaction |
| --- | --- | --- | --- |
| What is changing? | A few trend charts and noteworthy events | Previous-period comparison | Select a change and retain its time interval |
| Who is affected? | Filtered, sortable entity ledger | Distribution and map highlighting | Select the contributing population, not just its average |
| Why did this happen? | Recorded contribution breakdown | Related events, rules, and entities | Follow provenance links without losing context |
| When did it happen? | Multiscale timeline | Event details and state-at-time summaries | Brush an interval and synchronize other panels |
| How are these entities connected? | Scoped relationship graph | Searchable relationship table | Expand selected connections and expose omitted counts |

These are proposed workflows, not a claim that every observer should see all five views at once.

### Use a stable workspace with explicit context

A practical default is a collapsible watchlist on the left, the world view in the center, an entity inspector on the right, and an expandable analytical tray beneath it.

A persistent context strip should show:

**Selected scope · simulation date · inspected interval · live/pinned status · active filters**

For example:

**Eastbank households · Year 83, harvest season · previous 90 days · pinned · landless households**

Maintain one shared selection model across the 3D world and web panels. Selecting a household in a table should highlight its chart series and relevant timeline events. Moving the camera should remain an explicit “Locate” action; merely reading a record should not unexpectedly move the observer.

Preserve navigation history, pinned comparisons, column arrangements, and saved filters. Stable entity identifiers should continue to resolve after death, demolition, institutional dissolution, or renaming.

Most importantly, distinguish three operations:

**Pause the simulation** stops the world. **Freeze this view** stops its displayed updates. **Inspect a historical time** queries whatever historical records exist. Historical inspection must not imply full simulation rewind unless TCE actually supports it.

### Make ledgers stable under change

For TCE, a good dense table should provide persistent entity names, right-aligned tabular numerals, units in column headings, configurable precision, visible filter summaries, and copyable exact values. Use in-cell bars or sparklines only where they help comparison; do not turn every cell into a miniature dashboard.

**Live values should not require live reordering.** While a user is pointing at or navigating a row, preserve its position. Show updated values, accumulate a “ranking changed” indicator, and offer an explicit refresh of the ordering. Use a stable identifier as the final sort tie-breaker.

EVE’s Overview provides a concrete precedent: sorting can be frozen while new entries appear at the bottom and disappearing entries are visually distinguished. This allows interaction with a changing list without making targets constantly move. [EVE University Wiki](https://wiki.eveuniversity.org/Overview)

Filtering and sorting should operate over the complete query result, even when only a small viewport is rendered. Clearly distinguish “no matching entities,” “not recorded,” “still loading,” and a genuine numeric zero.

### Make charts answer comparisons, not merely display activity

For TCE, I would establish a small chart vocabulary:

| Data meaning | Default visual | Important safeguard |
| --- | --- | --- |
| Stocks: grain, population, treasury | Line or step chart | Identify whether values are instantaneous or period averages |
| Flows: births, production, migration | Period bars or rate lines | Show period length and rate denominator |
| Distribution: wealth, nutrition, age | Histogram, quantiles, or grouped distribution | Retain affected counts; do not substitute a mean |
| Comparison between settlements | Ranked dots/bars or small multiples | Use comparable units, periods, and scales |
| Recorded contributions | Signed contribution bars or waterfall | Distinguish accounting identity from causal explanation |
| Seasonality or repeated cycles | Aligned seasonal views or heatmap | Use the simulated calendar, not an assumed historical era |

Avoid a default chart containing thirty competing lines. Start with the selected entity, a meaningful reference, and perhaps a distribution band; provide explicit expansion.

A particularly useful vertical slice is **declining grain reserves**. The observer should move from the stock trend to production, consumption, trade, and loss components; then to affected households and relevant recorded decisions. Nearby events can suggest a hypothesis, but chronological proximity alone must not become a “why” statement.

Cohort definitions also matter. “Average household wealth increased” means something different when poor households emigrated. Offer both **current membership** and **fixed comparison cohort** where the history supports them.

### Give timelines semantic zoom

TCE needs more than a calendar widget. A daily activity trace, a lifetime, and two centuries of urban history require different representations.

At close zoom, show individual events and activity intervals. At intermediate zoom, group events into lanes such as household, institution, construction, and conflict. At distant zoom, show event density, long-lived institutional spans, and selected milestones.

Aggregation should remain inspectable: a cluster labeled “47 events” should open those events. A visually quiet century must not mean that unselected events were deleted.

Use interval bands for duration and point markers for instants. Virtualize both the time range and the visible lanes. Do not create one DOM element for every recorded event.

For TCE’s unscripted worlds, label periods using actual events and institutions—“Council of Eastbank,” “Northern canal construction”—rather than imposing authored historical eras.

### Make networks navigable before making them large

The default relationship view should be an ego network, family, institution, settlement, or selected trade network—not all 50,000 people.

Use separate relationship lenses for kinship, authority, trade, cooperation, and hostility. Show direction, relationship type, and time validity explicitly. When aggregating into groups, show group membership and hidden node/edge counts.

Preserve node positions while values change. Re-layout only when topology or the requested layout changes, and allow the observer to freeze the arrangement.

For dense group-to-group relationships, provide a matrix or ranked relationship table as an alternative. Rendering a graph successfully is not the same as making its structure understandable.

---

## 2. What worked and failed in comparable games

### Crusader Kings III: explain concepts without leaving the task

Paradox’s CK3 interface diary describes nested tooltips that let users follow linked game concepts. Tooltips can be locked through either a configurable timer or an explicit middle-mouse action. Its contextual tutorial opens relevant windows and highlights actual controls rather than relying solely on separate instructional text. The diary also distinguishes actionable alerts from less urgent suggestions. [Paradox Plaza Forum](https://forum.paradoxplaza.com/forum/threads/ck3-dev-diary-16-tutorials-and-tooltips-and-encyclopedias-oh-my.1345581/)

**Adopt:** contextual explanation, linked concepts, configurable tooltip persistence, and different urgency levels.

**Adapt:** give every substantial explanation a route into a persistent inspector. A tooltip chain is useful for a quick definition; it is less suitable for comparing several causes or reading a long institutional history.

**Do not copy blindly:** TCE is an observer simulation. “This is noteworthy” should not automatically become “you must fix this.” Separate interesting developments, operational errors, and observer-defined alerts.

The evidence here establishes the implemented design approach, not a measured improvement in learning time.

### Factorio: correct scope and metric definitions matter more than decoration

Factorio’s statistics redesign brought search and more compact arrangements into its statistics interface, including integrating exact values into thicker progress bars. [Factorio](https://factorio.com/blog/post/fff-337)

More revealing are the later semantic corrections. In Friday Facts #408, the developers explain that global production statistics became inadequate for optimizing individual planets and platforms, leading to per-surface statistics with an explicit global option. They also added a science total reflecting actual research output rather than merely science-pack consumption, and accumulator-charge history to answer whether stored energy survives the night. [Factorio](https://factorio.com/blog/post/fff-408)

The TCE equivalents are direct:

**Scope must be visible.** World food supply can be healthy while one settlement starves.

**The displayed metric must match the question.** Labor assigned is not labor completed; grain harvested is not edible food delivered; taxes assessed are not taxes collected.

**History must include the state needed for diagnosis.** Current storage capacity does not reveal whether reserves repeatedly reached zero.

Factoriopedia provides another useful lesson. The developers found that tooltip-contained relationships were insufficient for continued exploration, so they created linked, persistent pages, contextual opening, and back/forward navigation. Related technical concepts can be combined into one user-facing page even when represented separately internally. [Factorio](https://factorio.com/blog/post/fff-397)

For TCE, an observer should not need to understand the internal separation between a recipe, workshop type, technology node, and production capability before learning how iron tools are made.

### EVE Online: consistency helps, but padding can destroy useful density

CCP’s stated reasons for Photon included inconsistent interface styles and the maintenance cost of many divergent components. Its changes included a unified design language and compact modes. CCP reported **91% usage** after moving through an opt-out rollout; that is an adoption measure, not proof of satisfaction or faster task completion. [EVE Online](https://www.eveonline.com/news/view/improving-photon-ui)

A first-hand player comparison of Photon and the older interface criticized additional margins in the directional scanner because they reduced useful visible content and occupied more world-view space. The same analysis welcomed increased spacing in some menus. The lesson is not “all whitespace is bad,” but that density requirements differ between reading, targeting, navigation, and configuration. [the greybill](https://thegreybill.wordpress.com/2022/03/23/first-look-photon-ui/)

Subsequent official patch notes document adjustable window margins and headers, automatic UI scaling, multiple Overviews, and a scrollbar that no longer changes size on hover. These are specific examples of accommodating density and reducing moving interface geometry. [EVE Online](https://www.eveonline.com/news/view/patch-notes-version-20-11)

For TCE:

**Adopt a coherent component system, but expose density controls independently of text size.** Experts should be able to reduce padding without being forced to read smaller text.

**Keep interaction geometry stable.** Avoid expanding scrollbars, shifting headers, and automatic reordering under the pointer.

**Version saved filters.** EVE University warns that older Overview configurations can omit newly introduced categories. TCE’s authored building blocks create an analogous risk: a saved filter must not silently hide newly added institution or entity types. [EVE University Wiki](https://wiki.eveuniversity.org/Overview)

---

## 3. Which web libraries fit

### Recommended shortlist

The selection below is based on documented capabilities and TCE’s requirements, not a claim that one library wins every benchmark.

| Library | Relevant capability | Main cost or limitation | Recommendation for TCE |
| --- | --- | --- | --- |
| **Apache ECharts** | General analytical charting; Canvas and SVG rendering; data-driven updates through `setOption`. | Performance depends on chart type, configuration, and update strategy. Automatic chart updates do not replace upstream aggregation. | **Default chart engine.** Use Canvas for dense plots; evaluate SVG for small, annotation-heavy charts. [Apache ECharts](https://echarts.apache.org/handbook/en/best-practices/canvas-vs-svg/) |
| **uPlot** | Focused Canvas 2D time-series plotting; MIT licensed. | Data preparation and some interaction behavior remain application responsibilities. | Optional specialist for frequently updated time-series views after profiling. [GitHub](https://github.com/leeoniya/uPlot) |
| **Chart.js** | Canvas charts, documented decimation, and an OffscreenCanvas worker path. | Worker rendering complicates DOM-dependent plugins, interaction, configuration transfer, and resizing. | Credible alternative when the chart vocabulary is narrower; not an additional default dependency. [Chart.js](https://www.chartjs.org/docs/latest/general/performance.html) |
| **TanStack Table + TanStack Virtual** | Headless table logic plus separately composed virtualization. | Table does **not** include virtualization itself; TCE owns presentation and accessible interaction behavior. | **Default custom ledger stack.** Good fit for game-specific rows and inspectors. [TanStack](https://tanstack.com/table/v8/docs/guide/virtualization) |
| **AG Grid** | Integrated row/column virtualization and batched asynchronous transactions. | Less visual freedom than a headless approach; Community and commercial Enterprise capabilities differ. | Alternative when implementation speed and built-in grid behavior outweigh bespoke UI requirements. [AG Grid](https://www.ag-grid.com/javascript-data-grid/dom-virtualisation/) |
| **D3 utilities + custom Canvas** | Scales, interaction, and data-to-geometry tools without imposing a complete chart abstraction. | TCE must implement layout, hit testing, rendering, and accessibility. | **Long-term timeline approach**, where custom calendars and semantic zoom justify the work. [D3.js](https://d3js.org/what-is-d3) |
| **vis-timeline** | Points, ranges, groups, and zoom-dependent clustering. | Its standard timeline model still needs adaptation to TCE’s calendar, event semantics, and history scale. | Useful prototype or bounded entity-history widget; benchmark before making it the universal chronicle. [Vis.js](https://visjs.github.io/vis-timeline/docs/timeline/) |
| **Sigma.js + Graphology** | WebGL graph rendering with Graphology’s graph model and algorithms. | Rendering and layout are separate costs; readable labels and selection remain design problems. | Preferred specialist stack for scoped, read-oriented relationship exploration. [Sigma.js](https://www.sigmajs.org/docs/) |
| **Cytoscape.js** | Graph visualization with substantial styling, layout, and analysis capabilities. | Complex edges, labels, styles, and high pixel ratios can be expensive. | Alternative when rich graph semantics matter more than minimal rendering overhead. [Cytoscape.js](https://js.cytoscape.org/) |

### Important implementation distinctions

**Canvas versus SVG is not a simple fast-versus-slow choice.** ECharts’ guidance distinguishes large-mark workloads from many small chart instances; the renderer choice can also affect memory. Benchmark the actual mixture of chart sizes and counts. [Apache ECharts](https://echarts.apache.org/handbook/en/best-practices/canvas-vs-svg/)

**Virtualization reduces rendered elements, not necessarily data-processing work.** Sorting 50,000 records on every update is still work even when only forty rows are visible. Maintain indexes or sorted query results outside the row components, batch changes, and invalidate only affected views.

**WebGL does not make graph layout free.** Graphology’s ForceAtlas2 implementation offers a worker mode and optional Barnes–Hut optimization, which reduces the repulsion calculation from quadratic toward \(O(n\log n)\). That is not a guarantee about total layout or rendering time. [Graphology](https://graphology.github.io/standard-library/layout-forceatlas2.html)

I would initially ship **one general chart engine**, not separate libraries for every chart type. Add a specialist only when a measured bottleneck or a missing capability justifies its integration, accessibility, styling, and testing costs.

---

## 4. Real-time update architecture

### Publish read models, not the entire simulation

The proposed pipeline is:

`Committed Rust state → scoped query/read model → bounded update batches → worker processing → visible widgets`

A subscription should specify the entity scope, requested metrics, simulation-time range, aggregation, maximum output size, and desired delivery cadence.

Every response should carry enough metadata to interpret it correctly:

| Metadata | Why the panel needs it |
| --- | --- |
| World/session identifier and query generation | Reject responses from a previous world or superseded selection |
| Simulation tick and committed revision | Keep related panels consistent |
| Scope and population definition | Explain which entities contributed |
| Units and aggregation definition | Distinguish stocks, totals, means, and rates |
| Coverage and completeness | Identify missing intervals, partial buckets, or retained-history limits |

Generate shared data contracts from the Rust-side schema where practical, but still validate incoming data at the web boundary. TypeScript types alone should not be treated as proof that a runtime message is valid.

### Separate replaceable state from nonreplaceable events

Use two different delivery semantics.

**Current-state updates:** multiple pending changes to an entity’s displayed balance can often be replaced by the newest value. A bounded “latest state” mailbox is appropriate.

**Historical events:** births, deaths, office changes, enacted laws, and recorded causal decisions must not disappear because the panel could not keep up. Keep them in the simulation’s history system and retrieve them by cursor or range.

Retention and compaction should be explicit history policies. They should not emerge accidentally from a slow browser queue.

This distinction is especially important with WebSockets: the browser WebSocket API does not provide receive-side backpressure. An application that accepts messages faster than it processes them can accumulate memory or consume excessive CPU. Implement bounded queues, acknowledgements or delivery credits, and snapshot resynchronization at the application level. [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)

### Separate simulation time, delivery cadence, and interaction frames

A chart does not need to redraw once per simulation tick. At high acceleration, many simulated days may pass between useful display updates.

Use independent rates for:

**Simulation computation**, determined by the kernel.

**Evidence publication**, determined by panel needs and available resources.

**Interaction rendering**, which should remain responsive while the user pans, hovers, or selects.

When a view falls behind, retain the last valid contents and display its simulation timestamp and lag. Do not quietly present mixed revisions as one coherent current state.

Unsubscribe closed panels from expensive visual updates. A watchlist can retain lightweight summaries without keeping every hidden chart active.

### Aggregate history according to meaning

Use multiresolution histories and request a level suitable for the visible time range.

For stocks, retain appropriate endpoint values and, where useful, minimum/maximum envelopes. For flows, retain sums and elapsed durations. For rates, preserve the necessary numerator and denominator. Do not sum stocks or average already-averaged groups without their weights.

Missing observations should remain gaps. Incomplete current buckets should be visibly incomplete.

For rendering, use a point budget related to chart width. Chart.js documents a useful distinction: LTTB targets overall shape, while min–max decimation preserves extrema and can require up to four points per pixel. [Chart.js](https://www.chartjs.org/docs/latest/configuration/decimation.html)

For TCE, that suggests:

**Use extrema-preserving summaries where brief famine, price, mortality, or storage spikes matter. Use shape-oriented downsampling for exploratory trend browsing.**

Neither replaces an exact statistical query. The tooltip for a selected interval should be calculated from authoritative data or exact summaries, not by treating the displayed decimated points as the original dataset.

MinMaxLTTB research reports more than an order-of-magnitude acceleration of the downsampling step in its experiments by preselecting extrema before LTTB. That is promising for large historical ranges, but it is not an equivalent acceleration of the entire UI. [arXiv](https://arxiv.org/abs/2305.00332)

### Keep heavy work away from interaction handlers

Use workers for decoding, indexing, downsampling, and graph layout where supported. Transfer suitable numeric buffers rather than repeatedly cloning large object graphs.

Do not assume every chart library can simply be moved into a worker. Chart.js explicitly documents an OffscreenCanvas path, but also notes restrictions around transferable configuration, DOM-dependent plugins, interaction, and manual resizing. [Chart.js](https://www.chartjs.org/docs/latest/general/performance.html)

Keep chart instances alive and update their data instead of rebuilding them. Preserve zoom, selected series, and pinned comparisons across updates. ECharts’ documented dynamic-update mechanism is `setOption`; TCE should wrap it with stable series identity and explicit ownership of interaction state. [Apache ECharts](https://echarts.apache.org/handbook/en/how-to/data/dynamic-data/)

Workers also compete for the same machine resources as Rust and Unreal. Bound their workloads rather than treating them as free capacity.

---

## 5. Numbers: evidence, scaling arithmetic, and proposed budgets

### Published benchmark evidence

The uPlot maintainer’s **March 11, 2023** cold-start benchmark used a Ryzen 7 PRO 5850U, 32 GB RAM, Chrome 113 on Arch-based Linux, and device-pixel ratio 1.5:

| Library version in that benchmark | Reported “done” time | Peak / final JavaScript heap |
| --- | --- | --- |
| uPlot 1.6.24 | 34 ms | 21 / 3 MB |
| Chart.js 4.2.1 | 38 ms | 29 / 10 MB |
| ECharts 5.4.1 | 55 ms | 17 / 3 MB |

These are **maintainer-run, historical, workload-specific measurements**, not current library rankings, frame times, total process memory, or UE5 measurements. They justify testing candidates, not selecting a winner without testing. [GitHub](https://github.com/leeoniya/uPlot)

AG Grid provides another concrete implementation number: asynchronous transactions are batched with a **50 ms default waiting period**, avoiding repeated sort, filter, aggregation, and DOM work for individually applied changes. That is a batching mechanism, not a promise that all updates finish within 50 ms. [AG Grid](https://www.ag-grid.com/javascript-data-grid/data-update-high-frequency/)

### What TCE’s scale implies

The following are **derived examples using explicit assumptions**, not measurements:

| Hypothetical implementation | Arithmetic | Implication |
| --- | --- | --- |
| Render 50,000 rows with 20 columns | 1,000,000 cells | Do not construct the full ledger DOM |
| Render 40 visible rows, 8 overscan rows on each side, and 10 visible columns | 56 × 10 = **560 cells** | Virtualization changes rendering scale dramatically |
| Send a 128-byte summary for every person at 10 Hz | 50,000 × 128 × 10 = **64 MB/s**, before overhead | View-specific subscriptions matter more than micro-optimizing serialization |
| Keep ten 64-bit metrics daily for 50,000 people over 200 years | 50,000 × 10 × 365 × 200 × 8 = **292 GB** | The browser cannot be the complete historical archive |

The history example assumes constant population, 365-day years, and no compression; it excludes identifiers, timestamps, and other overhead. Its purpose is to expose the storage order of magnitude.

### Initial TCE performance targets

These are **proposed starting budgets**, to be revised against packaged-game measurements.

| Activity | Initial target |
| --- | --- |
| General ledger and summary refresh | 2–5 updates per second |
| Selected entity’s immediate state | Up to 5–10 updates per second when useful |
| Historical chart data refresh | 1–4 updates per second; interaction handled separately |
| Pointer tracking, crosshair, and panning | Aim for 60 Hz while interacting |
| Chart display density | Approximately 1–4 points per horizontal pixel, according to semantics |
| Default relationship expansion | Approximately 200–1,000 visible nodes as a **legibility default**, not a renderer limit |
| Pointer-response latency | Below 50 ms at the 95th percentile |
| Cached selection change | Below 100 ms at the 95th percentile |

At 60 Hz, a frame lasts approximately **16.7 ms**. Do not allocate that entire interval to web-panel work; Unreal’s rendering, input, and composition also need headroom.

---

## 6. Accessibility and UI scaling

### Separate density, text size, and rendering resolution

Provide independent controls for text scale, interface density, and overall panel arrangement.

A compact mode should first reduce unnecessary padding and decoration—not shrink essential text. Canvas charts must recalculate fonts, axes, and hit regions when scale changes; enlarging a previously rendered bitmap is not an adequate scaling implementation.

Microsoft’s game accessibility guidance gives PC text-size checkpoints of **18 pixels at 1080p and 36 pixels at 4K**, and recommends text scaling up to **200%** without losing content or functionality. Treat those as rendered-size checks; CSS pixels, device pixels, Windows scaling, and the embedded surface must be tested together. [Microsoft Learn](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/101)

For an initial design, I would use a comfortable text preset and allow compact row spacing separately. At larger text settings, collapse secondary columns into an expandable detail area rather than clipping content.

### Use explicit accessibility requirements

Relevant WCAG 2.2 targets include **4.5:1 contrast for normal text**, **3:1 for large text**, and **3:1 for necessary non-text interface distinctions**. The AA pointer-target criterion is **24 × 24 CSS pixels**, with specified spacing and other exceptions; 44 × 44 is the enhanced criterion, not the universal AA minimum. Text should support 200% resizing. Hover/focus content needs usable dismissal and persistence behavior. [W3C](https://www.w3.org/TR/WCAG22/)

For TCE, use opaque or sufficiently solid panel backgrounds so terrain brightness does not determine readability. Encode selection, relationship type, and warnings through more than color alone.

Avoid continuous flashes or animations for routine updates. Preserve a reduced-motion mode and a freeze-updates control.

### Provide nonvisual access to the evidence

Every chart should expose a concise summary, current selection, units, time range, and an accessible data-table route. ECharts supports generated descriptions and decal patterns, but its documentation also warns that simply describing every point can produce unhelpfully long output. TCE should supply meaningful summaries. [Apache ECharts](https://echarts.apache.org/handbook/en/best-practices/aria/)

Virtualized tables require particular care. ARIA row/column counts and indices can describe their structure, but rows absent from the DOM are not automatically available as ordinary readable content. AG Grid recommends pagination as one way to expose manageable pages without virtualization for screen-reader use. [W3C](https://www.w3.org/WAI/ARIA/apg/practices/grid-and-table-properties/)

Provide keyboard selection, column sorting, search, next/previous result, and a paginated accessible ledger mode. Preserve focus by entity identity during updates. Do not announce every changing numeric cell through a live region.

**Accessibility must be tested in the packaged UE host**, including its operating-system accessibility tree and focus routing. A page working correctly in standalone Chrome is not sufficient evidence that the embedded game interface works with Narrator or NVDA.

---

## 7. What TCE should adopt, adapt, and avoid

**Adopt immediately:** shared selection and time context; persistent inspectors; visible scope and metric definitions; virtualized ledgers; bounded subscriptions; explicit state-versus-event delivery; and saved investigative workspaces.

**Adapt carefully:** CK3’s nested explanations into pinnable provenance views; Factorio’s scoped statistics into settlement/cohort analysis; and EVE’s compact modes and stable interaction geometry into independently configurable density settings.

**Avoid:** full-ECS mirroring, continuous re-sorting under the pointer, one DOM element per historical event, unbounded browser history, auto-layout on every tick, charts whose averages conceal the affected population, and tooltip-only explanations.

### Build one complete investigative path before a chart gallery

The first integrated test should let an observer identify a grain shortage, locate affected households, inspect the accounting components, follow recorded contributing events, and return to the original view without losing selection or time context.

Then test that path with **50,000 agents and eight simultaneous panels**—for example, a ledger, four charts, a timeline, a graph, and an inspector—while the 3D city is active.

Measure input latency, browser main-thread work, worker utilization, transport queue depth, memory after warm-up, Unreal frame time, and kernel overhead. Test 1080p through 4K, multiple Windows scaling settings, event bursts, long histories, hidden-panel resumption, world changes, and missing-update recovery.

The degradation order should be explicit: reduce publication frequency, simplify display detail, and defer expensive layouts **before** compromising event completeness or silently changing metric meaning.

**The success criterion is not “the library can draw 50,000 objects.” It is “the observer can reliably explain a meaningful change while the simulation continues.”**

---

## 8. Source guide

The inline citations provide the evidence for individual claims. These are the most useful starting points for implementation and design review.

| Area | Sources and why they matter |
| --- | --- |
| Dense game statistics | [Factorio #337: Statistics GUI](https://factorio.com/blog/post/fff-337?utm_source=chatgpt.com), [#408: Statistics improvements](https://factorio.com/blog/post/fff-408?utm_source=chatgpt.com), and [#397: Factoriopedia](https://factorio.com/blog/post/fff-397?utm_source=chatgpt.com). Particularly useful for scope, metric definitions, and persistent exploration. |
| Contextual explanation | [CK3 Dev Diary #16](https://forum.paradoxplaza.com/forum/developer-diary/ck3-dev-diary-16-tutorials-and-tooltips-and-encyclopedias-oh-my.1345581/). Nested tooltips, configurable locking, tutorials, and alert hierarchy. |
| Density trade-offs | [CCP: Improving Photon UI](https://www.eveonline.com/news/view/improving-photon-ui?utm_source=chatgpt.com), [official 20.11 patch notes](https://www.eveonline.com/news/view/patch-notes-version-20-11?utm_source=chatgpt.com), and [The Greybill’s first-hand comparison](https://thegreybill.wordpress.com/2022/03/23/first-look-photon-ui/?utm_source=chatgpt.com). Read the developer and player perspectives together. |
| Live ledgers | [EVE University: Overview](https://wiki.eveuniversity.org/Overview?utm_source=chatgpt.com), [TanStack virtualization guide](https://tanstack.com/table/v8/docs/guide/virtualization?utm_source=chatgpt.com), and [AG Grid high-frequency updates](https://www.ag-grid.com/javascript-data-grid/data-update-high-frequency/?utm_source=chatgpt.com). |
| Chart performance | [uPlot benchmark and source](https://github.com/leeoniya/uPlot?utm_source=chatgpt.com), [ECharts renderer guidance](https://echarts.apache.org/handbook/en/best-practices/canvas-vs-svg/?utm_source=chatgpt.com), and [Chart.js performance guidance](https://www.chartjs.org/docs/latest/general/performance.html?utm_source=chatgpt.com). |
| Downsampling research and talk | [MinMaxLTTB paper](https://arxiv.org/abs/2305.00332?utm_source=chatgpt.com) and [IEEE VIS presentation](https://www.youtube.com/watch?v=jyJJK4Xl8OE&utm_source=chatgpt.com). Relevant to long, interactive simulation histories. |
| Networks and timelines | [Sigma documentation](https://www.sigmajs.org/docs/?utm_source=chatgpt.com), [Graphology ForceAtlas2](https://graphology.github.io/standard-library/layout-forceatlas2.html?utm_source=chatgpt.com), [Cytoscape.js documentation](https://js.cytoscape.org/?utm_source=chatgpt.com), and [vis-timeline documentation](https://visjs.github.io/vis-timeline/docs/timeline/?utm_source=chatgpt.com). |
| Accessibility | [WCAG 2.2](https://www.w3.org/TR/WCAG22/?utm_source=chatgpt.com), [Xbox Accessibility Guideline 101](https://learn.microsoft.com/en-us/xbox/accessibility/xbox-accessibility-guidelines/101?utm_source=chatgpt.com), and [AG Grid accessibility guidance](https://www.ag-grid.com/javascript-data-grid/accessibility/?utm_source=chatgpt.com). |

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9299b-6f00-83e9-99e8-e5451e7e8d62)
