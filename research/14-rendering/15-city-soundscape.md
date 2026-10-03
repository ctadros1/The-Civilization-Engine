# Procedural city soundscapes for The Civilization Engine

**Recommendation:** Build audio as a **bounded, listener-centered representation of the simulation**, not as one sound emitter per person. Use Unreal’s Audio Mixer, a small library of parameterized MetaSounds, and a C++ subsystem that turns Rust’s activity data into district ambience, representative local sources, and a limited number of exact events.

For TCE, the essential distinction is:

> **The simulation determines what is happening. The audio system determines which parts the listener can meaningfully hear.**

Soundscape is useful for supplementary ambient spawning, but I would not make it the foundation of TCE’s activity audio. Full acoustic simulation should be a later, optional enhancement.

**Version scope:** This report is current to **September 27, 2026**. Epic released UE 5.8 on June 23, 2026; the engine documentation referenced below identifies itself as UE 5.8. Older game postmortems and papers are architectural precedents, not demonstrations of present-day UE 5.8 performance. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

---

## 1. Options: what to build with

### 1.1 The main approaches

These approaches are complementary rather than mutually exclusive.

| Approach | How it works | Strengths | Limitations and fit for TCE |
| --- | --- | --- | --- |
| **Sound Waves/Sound Cues with a C++ scheduler** | Code selects recordings, positions sources, and controls their playback. | Straightforward baseline; good for footsteps, impacts, doors, and simple loops. | Does not solve population scaling by itself. Use for sounds that do not need substantial internal procedural behavior. |
| **Parameterized MetaSounds** | A reusable audio graph receives activity parameters and generates or combines audio accordingly. | Good for continuously changing crowds, machinery, weather, and randomized variation. | Graph complexity, internal sample playback, and effects still cost processing time. **Best primary authoring system for TCE.** |
| **Soundscape plugin** | State-driven ambient spawning using sound definitions and collections of those definitions. | Convenient environmental variation without hand-placing every emitter. | Still marked **Beta** in UE 5.8. Use behind an adapter for optional ambience, not as the owner of authoritative work or social events. |
| **Middleware, particularly Wwise** | A separate audio-authoring and runtime layer receives game data and controls the soundscape. | Relevant shipped-game precedent, particularly *Planet Coaster*. | Adds another integration, content-build pipeline, and version dependency. It does not eliminate the need for TCE’s aggregation system. |
| **Acoustic propagation plugins** | Calculate how existing sounds travel through geometry, including occlusion, reflections, and transmission. | Can substantially improve enclosed streets and interiors. | These are **propagation systems, not city-sound generators**. Introduce only after the underlying soundscape works and is profiled. |

UE’s Sound Cue and MetaSound documentation supports the native approaches; Soundscape explicitly retains its Beta warning. Frontier’s *Planet Coaster* articles document a separate, data-informed crowd-audio representation rather than simply attaching audible voices to every guest. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/sound-cues-in-unreal-engine)

The most valuable procedural work is likely to be **selection, timing, layering, and parameterization of good recordings**. TCE does not initially need physical synthesis of every hammer, animal, or conversation.

### 1.2 What MetaSounds should do

MetaSounds provide audio-rate processing and sample-accurate trigger timing. A **Source** is playable; a **Patch** contains reusable graph logic; a **Preset** inherits a graph while overriding inputs. Runtime-adjustable inputs and construction-time inputs have different purposes—do not assume every graph value is intended for continuous external updates. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/metasounds-reference-guide-in-unreal-engine)

I would begin with four graph families. These names and parameters are proposed TCE interfaces, not built-in Unreal assets.

| Proposed graph | Inputs from TCE’s presentation layer | Behavior |
| --- | --- | --- |
| `MS_CrowdCell` | Effective population, movement, social activity, agitation, cultural repertoire, seed | Crossfades crowd textures and schedules occasional local vocal details. |
| `MS_Worksite` | Active state, material, work intensity, machine speed, condition, seed | Produces work rhythms and continuous components appropriate to the actual process. |
| `MS_Environment` | Wind, precipitation, water activity, habitat, season, time of day | Controls environmental beds and sparse ecological details. |
| `MS_LandmarkSignal` | Instrument family, pattern, urgency, institution, trigger | Plays identifiable bells, drums, horns, or other signals associated with actual events. |

Keep graphs bounded. A single MetaSound containing many simultaneous sample players still performs their decoding and processing. Combining everything into one graph also does not automatically preserve a separate world-space position for each internal layer. Budget **internal playback and DSP**, not merely the number of visible graph assets. Unreal’s mixer documentation distinguishes source generation, decoding, source processing, and mixing costs. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/audio-mixer-overview-in-unreal-engine)

### 1.3 Where Soundscape fits

Soundscape’s **Colors** describe sounds and playback behavior; **Palettes** group Colors under activation conditions. Gameplay Tags can activate states, and Colors can play MetaSound Sources. Its controls include randomized spawning and playback variation. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/soundscape-quick-start)

For TCE, use that for such things as wind through an inhabited district, scattered wildlife, or nonessential environmental detail.

Do **not** let a generic industrial palette invent hammering in an abandoned workshop. Worksite activity should come from the kernel, with an exact or representative source placed at a real active location. Soundscape may render an additional diffuse layer, but should not decide that the work exists.

---

## 2. Trade-offs and performance

### 2.1 The scaling problem is representation, not sample playback

The wrong starting point is:

```
50,000 people → 50,000 persistent AudioComponents → distance attenuation
```

The recommended starting point is:

```
50,000 simulated people
    → incrementally maintained activity summaries
    → nearby audible regions and important events
    → a bounded set of sound sources
```

The distant crowd should exist as **aggregate state**, not as thousands of silent sound objects.

This approach has research precedent. Tsingos and colleagues developed perceptual culling and source clustering so complex environments could be represented using a limited number of spatialized representatives. The useful principle is to spend expensive spatial processing on perceptually important sources—not to reproduce their early-2000s hardware implementation. [SOP Inria](https://www-sop.inria.fr/reves/Nicolas.Tsingos/publis/RR-4734.pdf)

### 2.2 What published benchmarks actually establish

I found **no transferable benchmark** showing that a particular MetaSound crowd implementation meets TCE’s target on a 13th-generation i9 with UE 5.8. No TCE runtime was measured for this report.

A useful research comparison comes from Schissler and Manocha’s multi-source acoustic-propagation work. Their author manuscript reports the following on a **3.5 GHz, four-core CPU**:

| Research scene | Original sources | Resulting clusters | Clustering time | Propagation time |
| --- | --- | --- | --- | --- |
| Outdoor city | 50 | 24 | 0.08 ms | 47.2 ms |
| Indoor tradeshow | 200 | 95 | 0.21 ms | 182.7 ms |

These are timings from a separate research pipeline, **not Unreal frame times or audio-callback budgets**. The system runs propagation and rendering concurrently. The practical lesson is that reducing sources can be inexpensive relative to computing elaborate propagation; “interactive acoustics” must not be read as “fits comfortably inside a 60 fps game.” [Gamma](https://gamma.cs.unc.edu/MULTISOURCE/paper.pdf)

### 2.3 Proposed initial budgets for TCE

The following are **prototype targets to test**, not measured engine limits or historical parameters.

| Resource | Initial target | Rationale |
| --- | --- | --- |
| Active city-sound sources | **64 maximum** | Enough room for layered ambience and selected details without tying cost to population. |
| Total project source allowance | **96 initially** | Leaves 32 outside the city allocation for UI, music, important transients, and transitions. Verify actual platform limits. |
| Expensive binaural/spatialized sources | **Up to 16** | Reserve for nearby, readily localizable details. |
| Audible-region selection | **5–10 updates/s** | Smooth the resulting parameters rather than rebuilding the soundscape every frame. |
| Detailed occlusion candidates | **Up to 16 at 5 updates/s** | Approximately 80 checks/s for this layer; stagger them across frames. |
| Audio-management game-thread work | **p95 below 0.5 ms/frame** | A starting allowance for selection, parameter submission, and lifecycle management. |
| Audio-render callback | **p99 below 2 ms at 48 kHz/512 samples** | An initial headroom target; also inspect worker dependencies and underruns. |
| Audio working memory | **Start with a 256 MiB target** | Include decoded data, streaming cache, graph state, and effects—not just compressed asset sizes. |

An illustrative allocation of the 64 city sources is **24 close details, 16 neighborhood representatives, eight district layers, four environmental layers, four landmark sources, and eight transient slots**. This is a budget allocation, not a requirement to keep all slots occupied.

Audio and graphics have different deadlines. At 60 fps, a graphics frame lasts approximately **16.67 ms**. At 48 kHz, a 512-sample audio block lasts approximately **10.67 ms**; 1,024 samples lasts **21.33 ms**. These are derived block durations, not complete end-to-end latency. Unreal explicitly describes buffering as a trade-off between latency and protection from underruns. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/audio-mixer-overview-in-unreal-engine)

### 2.4 The important UE pitfalls

**Concurrency is a guardrail, not the aggregation algorithm.** Unreal’s concurrency rules count active components, including components that are not currently audible. Source Buses can consume two concurrency slots. Use global/category concurrency assets to catch bursts, but prevent irrelevant emitters from being created in the first place. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/sound-concurrency-reference-guide)

**`PlayWhenSilent` is not a cheap virtual voice.** UE 5.8 documents that it continues using a voice. `Restart` restarts a looping sound when realized; `SeekRestart` attempts to resume at the appropriate point, but is marked **Experimental**. Test the chosen mode rather than inferring behavior from the word “virtualization.” [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/EVirtualizationMode)

**Finish one-shots correctly.** A MetaSound one-shot that fails to signal completion can remain active indefinitely. This is particularly damaging in an endless simulation. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/metasounds-reference-guide-in-unreal-engine)

**Separate disk size from runtime memory.** As arithmetic, one minute of stereo audio at 48 kHz occupies **11.52 MB at 16-bit PCM**, or **23.04 MB at 32-bit float**. Unreal’s mixer operates on decoded floating-point data. Keep latency-sensitive, frequently reused sounds readily available; use controlled streaming for long beds rather than making the entire library resident. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/audio-mixer-overview-in-unreal-engine)

On the specified hardware, I would initially spend the RTX 4070 Ti’s budget on rendering. Do not assume the GPU or its VRAM makes elaborate audio graphs inexpensive; investigate GPU-assisted acoustics only as a separately measured feature.

---

## 3. Precedents and what to borrow

| Precedent | What is documented | Lesson for TCE |
| --- | --- | --- |
| **Epic City Sample / Soundscape, 2022** | Epic describes Soundscape as developed for City Sample, using dynamic data to support the city’s moving population and vehicles. This is a sample, not a shipped city-simulation benchmark. | Start from procedural world data rather than manually positioned ambience. Inspect the implementation, but keep a small TCE-facing adapter. |
| **Planet Coaster, 2016; developer articles, 2017** | Frontier’s crowd work describes a Soundbox representation informed by crowd density and other game data, with additional audio layers and representative emitters. | The closest conceptual precedent: a simulation can contain many individuals while audio operates on a separate, bounded representation. |
| **Fortnite’s Rave Cave, discussed in 2022** | Epic’s audio talks cover Audio Buses and spatial submix techniques in a shipped game. | Native Unreal audio can support complex spatial mixing. This does not establish a 50,000-agent capacity. |
| **Anno 1800** | Ubisoft’s audio discussion describes an environmental sound system that adapts to camera zoom and movement across the map. | Camera behavior deserves explicit sound-design rules; ordinary distance attenuation is not a complete strategy-game listener model. |
| **Tsingos et al., 2003/2004** | Perceptual culling, clustering, and representative spatial sources. | Use audibility and perceptual importance to allocate processing. |
| **Scaper, 2017 research and open-source implementation** | Python tooling synthesizes soundscapes from foreground/background events and probability distributions, with event annotations. | Useful for offline test fixtures and repeatable content experiments—not as TCE’s runtime renderer. |

The first and third examples are documented in Epic’s GameSoundCon summary; Frontier’s original series describes the crowd abstraction; Ubisoft’s discussion establishes the camera-dependent behavior. The research and Scaper sources are independent implementations rather than Unreal plugins. [Unreal Engine](https://www.unrealengine.com/blog/get-game-audio-tips-from-our-favorite-talks-at-gamesoundcon)

### Acoustic middleware: useful, but a separate decision

**Steam Audio** is the stronger open-source propagation candidate to investigate later. Its repository identifies **version 4.8.1**, uses Apache 2.0 licensing, and includes an Unreal integration. The documented capabilities include HRTF rendering, occlusion/transmission, reflections, and dynamic geometry. However, the repository’s broad “UE 4.27+” compatibility statement is **not proof that a particular release has been validated against TCE’s UE 5.8 configuration**. [GitHub](https://github.com/ValveSoftware/steam-audio)

There is an important geometry constraint: the documented Unreal dynamic-object workflow supports reusable exported geometry moving as a rigid object. It does not imply automatic handling of arbitrary runtime mesh-topology changes. Runtime-assembled buildings require an explicit geometry synchronization design. [Valve Software](https://valvesoftware.github.io/steam-audio/doc/unreal/guide.html)

I would not start a new TCE dependency on **Project Acoustics**. Its public repository was archived in July 2024, and its documented baked-world workflow is a poor default for a city whose geometry keeps changing. That repository status should not be confused with a claim that all Microsoft acoustic research or technology was discontinued. [GitHub](https://github.com/microsoft/ProjectAcoustics)

---

## 4. Recommended TCE architecture

### 4.1 Keep authority in Rust and audio presentation in Unreal

Use a dedicated C++ world subsystem—conceptually `UTCEAudioWorldSubsystem`—to own listener policy, emitter selection, pooling, and parameter submission.

Rust should expose **two kinds of information**:

| Data stream | Contents | Purpose |
| --- | --- | --- |
| **State snapshots/deltas** | Stable cell/site IDs, location, active workers by process, pedestrian/animal/vehicle flow, social activity, environmental conditions, acoustic-zone revision | Reconstruct what should currently be audible, including after load, teleport, or a dropped update. |
| **Sparse semantic events** | Stable sequence number, simulation time, event type, source ID/location, intensity, significance | Represent collapses, alarms, starts/stops, celebrations, and other distinctive occurrences. |

Do not send a cross-language call for every footstep or hammer strike. Maintain broad activity summaries incrementally as simulation state changes; let nearby presentation objects supply precisely synchronized detail where required.

The DLL boundary should use a **versioned C ABI**, fixed-width fields, explicit buffer ownership, and compatible layouts. Do not expose Rust collections or Unreal objects across it, and do not permit uncontrolled unwinding across the boundary. Rust’s FFI guidance documents these layout, ownership, callback, and unwinding concerns. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

For TCE specifically:

* Transfer bounded batches to the presentation layer; never block the audio-render thread waiting for Rust.
* Keep audio random-number generation separate from simulation randomness.
* Translate kernel coordinates to Unreal coordinates explicitly, including units and origin changes.
* Make state updates recoverable. A lost decorative chirp is acceptable; a permanently stuck industrial loop is not.

An essential rule is that **muting audio or moving the camera must not alter history**. Any gameplay consequences of noise—waking people, attracting guards, signaling assemblies—belong in the kernel’s own model, not in the presentation mixer.

### 4.2 Implement hierarchical audio LOD

Use coarse cells initially, with optional street or room connectivity later. These are proposed starting distances for ordinary local detail, **not universal acoustic audibility limits**.

| Representation | Illustrative range | What to render |
| --- | --- | --- |
| **Exact local detail** | 0–20 m | Selected nearby footsteps, visible tool contacts, doors, conversations, animals. |
| **Neighborhood representatives** | 20–100 m | A few sources representing active markets, workshops, movement flows, or groups. |
| **District texture** | 100–500 m | Diffuse crowd/work/transport layers, with sparse recognizable signals. |
| **Distant environment** | Beyond ordinary detail range | Weather, broad landscape ambience, and exceptional landmarks—not individual footsteps. |

Loud bells, waterfalls, major machinery, and similar sources need their own reach and priority rules. Do not force all categories into the same distance bands.

Use **hysteresis and crossfades** when changing representation. An initial experiment could use 20–30% boundary hysteresis and 0.25–1 second transitions for local representation changes, with longer smoothing for district textures.

Avoid three common mistakes:

**Do not cull by the visual frustum.** Something behind the camera may still be prominent acoustically.

**Do not continuously drag a representative emitter between unrelated sources.** Anchor representatives to stable cells, sites, or street segments. Fade between them when selection changes.

**Do not count the same population twice.** As individual details become prominent, reduce the corresponding diffuse contribution rather than adding them on top of an unchanged full-density bed.

For selection, rank candidates by predicted audibility, semantic importance, current focus, and persistence. Give major signals a reserved allocation so they cannot be crowded out by footsteps.

### 4.3 Crowd murmur must reflect activity, not just population

A convincing crowd layer needs at least three scales:

**Diffuse texture:** overlapping, largely unintelligible social sound appropriate to the number and behavior of people.

**Small-group details:** occasional exchanges, laughter, calls, reactions, and movement sounds.

**Exact close events:** a few audible individuals tied to actual nearby activity.

A useful proposed density mapping is:

\[
d=1-e^{-N\_{\mathrm{active}}/N\_0}
\]

Here, \(N\_{\mathrm{active}}\) is the effective contributing population and \(N\_0\) is an authored calibration value. Use \(d\) to blend texture families and event density—not simply to multiply volume.

A crowd of silent listeners should not sound like an equally large market. Include social state: conversing, waiting, listening, celebrating, arguing, fleeing, or sleeping.

For decorative details, use bounded stochastic scheduling:

\[
P(\text{event during }\Delta t)=1-e^{-\lambda \Delta t}.
\]

The rate \(\lambda\) can rise with active population but should saturate. Add minimum gaps and category limits. Exact visible impacts should follow presentation synchronization instead of this stochastic schedule.

Record or author variation across **cadence, vocal intensity, group size, phrase length, and response patterns**. Pitch randomization alone will not provide enough variety for a simulation experienced over hundreds of hours. Keep each culture’s repertoire internally coherent; avoid inserting conspicuously modern conversations into an early agrarian town.

### 4.4 Make activity causally audible

The audio system should be able to explain every prominent sound:

| Simulation state | Proposed audible consequence |
| --- | --- |
| A workshop has workers, materials, and an active process | Its relevant worksite graph runs. |
| Workers leave, fuel runs out, or production stops | The process winds down; unrelated ambient sound may remain. |
| Traffic changes from pedestrian movement to carts | Footfall texture changes and wheel/axle/load sounds appear. |
| A worksite changes material or tool technology | Its sample/synthesis family and rhythm change. |
| A market loses attendance | Crowd density and vendor-call frequency decline. |
| A scheduled assembly actually occurs | Crowd behavior and the appropriate institutional signals change. |
| A building collapses | One authoritative presentation event coordinates sound with the visible collapse. |

Close visible activity needs a **single synchronization owner**. For example, a worker’s rendered animation may issue an impact cue derived from a kernel-authorized work action. Do not also play the same impact independently from a simulation notification.

At greater distance, approximate the same process with a representative rhythmic source. Maintain local rhythm, but vary phase and cadence between unrelated workers so an entire district does not become synchronized.

### 4.5 Historical soundscapes: use technological and institutional conditions, not era loops

Preindustrial cities should not automatically sound pastoral. Seneca’s *Letter 56* describes a noisy urban environment involving baths, exertion, splashing, hawkers, traffic, and craft sounds. It establishes the presence and variety of those activities—not a calibrated historical decibel distribution. [Wikisource](https://en.wikisource.org/wiki/Moral_letters_to_Lucilius/Letter_56)

For TCE, the following is a **proposed palette organized by historical analogues**, not a universal chronology:

| Historical analogue | Candidate sound families | Conditions that should enable them |
| --- | --- | --- |
| **Early agrarian settlement** | Food preparation, hand processing, domestic animals, footsteps, fires, seasonal field work, local ecology | Actual population, livestock, work schedule, weather, and habitat. |
| **Ancient or other preindustrial urban center** | Dense trade, craft districts, public gatherings, carts, water handling, baths where present | Urban density, institutions, transport technology, paving, and operating workplaces. |
| **Institutionally organized preindustrial town** | Proclamations, processions, collective ritual, bells or other signaling instruments | A society’s specific institutions, instruments, conventions, and scheduled events. |
| **Mechanized preindustrial economy** | Mills, pumps, repetitive powered processes, heavier material handling | Working mechanisms, available power, production load, and maintenance condition. |
| **Steam/industrial development** | Engines, industrial machinery, rail or steam transport, factory signals | Actual adoption and operation of the relevant technologies. |
| **Electrified or motorized development** | Motor traffic, electrical machinery, amplified sound | Infrastructure, machines, operating schedules, and regulation—not an era flag. |

Institutional signals deserve particular care. Niall Atkinson’s research on Florence describes bells as part of civic and religious organization, including their political use during the Ciompi uprising. They were meaningful signals within a learned social system, not merely background decoration. [earlymoderncommunities](https://earlymoderncommunities.org/home/interviews-2/niall-atkinson/)

Accordingly, model an instrument’s **ownership, function, pattern, and occasion**. Different societies may use different signal systems; do not universally attach European church bells to “medieval” development.

There is also a distinction between historical evidence and reconstruction. A 2020 study reconstructed vendor cries in Istanbul and Naples from written evidence, comparing a DAW-based approach with UE4–Wwise. Its relevance is the evidence-to-content process, not a guarantee that the reconstructed sound is an exact recording of the past. [Istanbul Technical University](https://research.itu.edu.tr/en/publications/the-soundscape-reconstructions-of-the-early-20supthsup-century-ve/)

The **Virtual Paul’s Cross Project** provides another useful precedent: a reconstructed 1622 sermon can be heard from eight positions with four crowd sizes. It illustrates why audience density, listening position, and urban geometry can matter as much as the selected voice recording. [Virtual Pauls Cross Website -](https://vpcp.chass.ncsu.edu/)

For every historical sound family, store provenance and confidence separately from its implementation. “Well-attested activity, plausible recording substitute” is more honest and useful than treating all authored audio as equally authentic.

### 4.6 Define camera and simulation-time policies explicitly

TCE needs two listener modes or a deliberate blend between them.

**Street observation:** spatial detail follows a plausible listener near the camera.

**Strategic overview:** the soundscape focuses on the district under observation and becomes more diffuse as the view rises. This is an intentional sound-design abstraction, not literal acoustics at the camera’s altitude.

Do not merely move a physically attenuated listener hundreds of meters upward and accept either silence or an unnaturally amplified entire city.

Simulation speed needs similar treatment:

| Situation | Recommended policy |
| --- | --- |
| Normal play | Render activity and nearby events normally. |
| Fast simulation | Update activity composition faster, but keep ordinary audible cadence in real time. |
| Large time jump | Replace state with a short transition; discard obsolete minor events. |
| Many important events | Summarize or prioritize; retain complete information in the event log. |
| Pause | Apply an explicit rule: freeze human activity, while optionally retaining a quiet environmental bed. |

**Do not multiply playback pitch or event rate by the simulation speed.** A thousandfold time acceleration should not produce a thousand hammer strikes per audible second.

### 4.7 Start with inexpensive acoustic structure

For the initial implementation, I recommend distance attenuation, a small number of shared reverberation environments, and occlusion only for selected important sources. Unreal’s attenuation system exposes spatialization, filtering, occlusion, and reverb controls; the native mixer describes ray-trace-based occlusion rather than a complete diffraction solver. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/sound-attenuation-in-unreal-engine)

Use TCE’s building metadata to distinguish open terrain, street canyons, covered markets, workshops, and interiors. Approximate acoustic connectivity with rooms and openings when needed. That can produce useful differences without simulating every reflection from every modular mesh.

Keep reverberation shared where appropriate. Reserve specialized source processing for sounds whose identity or location genuinely benefits from it.

### 4.8 A practical solo-developer implementation sequence

**First playable slice:** one settlement, a bounded emitter pool, basic activity summaries, the four graph families, and a debug overlay explaining why each source exists. Establish empty-street silence, operating-worksite correctness, and camera transitions before adding a large library.

**Second slice:** cultural palettes, better crowd states, streaming policy, landmark signaling, simple acoustic zones, and save/load recovery.

**Third slice:** only after profiling and listening tests, investigate more sophisticated spatialization or Steam Audio.

For AI coding agents, keep the source of truth in text: parameter schemas, palette manifests, tests, and C++ policy code. Generate or validate repetitive assets from those definitions where practical. The MetaSound Builder API supports programmatic graph construction, but remains **Beta**, with documented limitations; use it first for controlled authoring workflows rather than continually rebuilding graphs during gameplay. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/metasound-builder-api-in-unreal-engine)

### 4.9 Acceptance tests

Profile **packaged Development builds**, not only editor sessions. Epic documents differences in editor audio-thread behavior, while Audio Insights supports both PIE and standalone monitoring of sources, buses, submixes, and parameters. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/audio-mixer-overview-in-unreal-engine)

The test matrix should include:

| Test | Required outcome |
| --- | --- |
| 10,000 versus 50,000 people, same nearby activity | Similar audio-render cost; global population alone must not increase active voices. |
| Dense market or festival | Richer texture without exceeding category and global budgets. |
| Camera flight and teleport | No backlog burst, repeated restart clicks, or migrating phantom emitters. |
| Worksite shutdown and demolition | No orphaned loops or stale acoustic zones. |
| Maximum simulation speed and time jumps | Stable audible cadence; obsolete minor events discarded. |
| Cold asset cache and combined simulation/render load | No audible starvation or callback underruns. |
| Long-session soak | Stable component count, memory, and source lifecycle; acceptable repetition. |

Capture frame-time percentiles, audio-callback timing, active sources, internal playback workload where observable, memory, and underruns. Automated tests should also verify semantic rules: no activity sound from an inactive site, no duplicate collapse event, and no change to kernel outcomes when audio is disabled.

Add separate category volumes, a reduced-dynamic-range option, and visual equivalents for important signals. The soundscape should improve observation without becoming the only way to understand an event.

---

## 5. Sources, links, and version applicability

### Unreal documentation

| Source | Applicability |
| --- | --- |
| [UE 5.8 release announcement](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available?utm_source=chatgpt.com) | Establishes the June 2026 release baseline. |
| [MetaSounds reference](https://dev.epicgames.com/documentation/en-us/unreal-engine/metasounds-reference-guide-in-unreal-engine?utm_source=chatgpt.com) and [Builder API](https://dev.epicgames.com/documentation/en-us/unreal-engine/metasound-builder-api-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8 documentation; Builder remains Beta. |
| [Soundscape overview](https://dev.epicgames.com/documentation/en-us/unreal-engine/soundscape-in-unreal-engine?utm_source=chatgpt.com) and [quick start](https://dev.epicgames.com/documentation/en-us/unreal-engine/soundscape-quick-start?utm_source=chatgpt.com) | UE 5.8; Beta ambient-generation plugin. |
| [Concurrency reference](https://dev.epicgames.com/documentation/en-us/unreal-engine/sound-concurrency-reference-guide?utm_source=chatgpt.com) and [`EVirtualizationMode`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/EVirtualizationMode?utm_source=chatgpt.com) | UE 5.8 lifecycle and voice-management behavior. |
| [Audio Mixer overview](https://dev.epicgames.com/documentation/en-us/unreal-engine/audio-mixer-overview-in-unreal-engine?utm_source=chatgpt.com), [attenuation](https://dev.epicgames.com/documentation/en-us/unreal-engine/sound-attenuation-in-unreal-engine?utm_source=chatgpt.com), and [Audio Insights](https://dev.epicgames.com/documentation/en-us/unreal-engine/audio-insights-in-unreal-engine?utm_source=chatgpt.com) | UE 5.8 documentation; architecture, propagation approximations, and profiling. |

### Talks, research, and code

| Source | Applicability |
| --- | --- |
| [Epic’s GameSoundCon talks summary](https://www.unrealengine.com/blog/get-game-audio-tips-from-our-favorite-talks-at-gamesoundcon?utm_source=chatgpt.com) | 2022 City Sample, Soundscape, and Fortnite precedents; not a 5.8 benchmark. |
| Planet Coaster: [data-driven crowd management](https://www.audiokinetic.com/en/community/blog/planet-coaster-part-1-crowd-management-using-data-to-generate-dynamic-crowd-audio/?utm_source=chatgpt.com), [Soundbox](https://www.audiokinetic.com/en/community/blog/planet-coaster-part-2-crowd-audio-the-crowd-soundbox-system/), and [additional layers](https://www.audiokinetic.com/en/community/blog/planet-coaster-crowd-audio-additional-layers-part-3/?utm_source=chatgpt.com) | Original developer series, 2017; highly relevant architecture, not a native-UE implementation. |
| [Tsingos et al., perceptual audio rendering](https://dl.acm.org/doi/10.1145/1015706.1015710?utm_source=chatgpt.com) | SIGGRAPH/TOG 2004; enduring culling and spatial-LOD concepts, obsolete hardware context. |
| [Schissler–Manocha project, paper, and video](https://gamma.cs.unc.edu/MULTISOURCE/?utm_source=chatgpt.com) | Author project dated 2016; the benchmark discussed above comes from its manuscript. |
| [Steam Audio code](https://github.com/ValveSoftware/steam-audio?utm_source=chatgpt.com) and [Unreal guide](https://valvesoftware.github.io/steam-audio/doc/unreal/guide.html?utm_source=chatgpt.com) | Repository identifies 4.8.1; validate the exact UE 5.8 integration before adoption. |
| [Project Acoustics repository](https://github.com/microsoft/ProjectAcoustics?utm_source=chatgpt.com) | Archived repository; reference material rather than a recommended new dependency. |
| [Scaper code and paper links](https://github.com/justinsalamon/scaper?utm_source=chatgpt.com) | Open-source offline soundscape synthesis; original research published in 2017. |
| [Rust FFI guidance](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | Relevant to TCE’s DLL boundary, independent of audio middleware. |

### Historical evidence and reconstruction

[Seneca, Letter 56](https://en.wikisource.org/wiki/Moral_letters_to_Lucilius/Letter_56?utm_source=chatgpt.com) provides primary literary evidence of ancient urban sound. [Niall Atkinson’s Florence discussion](https://earlymoderncommunities.org/home/interviews-2/niall-atkinson/?utm_source=chatgpt.com) explains civic and religious signaling. [Virtual Paul’s Cross](https://vpcross.chass.ncsu.edu/?utm_source=chatgpt.com) demonstrates historically informed acoustic reconstruction. The [Istanbul–Naples vendor-cries study](https://research.itu.edu.tr/en/publications/the-soundscape-reconstructions-of-the-early-20supthsup-century-ve/?utm_source=chatgpt.com) documents a 2020 comparison involving UE4–Wwise.

**Bottom line:** TCE’s first audio milestone should be a city that becomes audibly different when its people actually change what they are doing. A bounded native-UE system can pursue that goal without making audio scale with all 50,000 individuals—and without committing the project to expensive physical acoustics before the soundscape itself is convincing.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9298f-e9d0-83e9-9caf-ff998d0b863a)
