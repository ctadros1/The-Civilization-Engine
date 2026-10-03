# LLM-driven societies: an engineering report for TCE

**Research cutoff: September 27, 2026.** Published measurements, cost calculations, and proposed TCE budgets are distinguished below.

## Executive recommendation

**Build the LLM layer as an optional, asynchronous proposer of long-lived intentions—not as the simulation engine, the keeper of personal identity, or the executor of everyday actions.**

For TCE, the useful division is:

> **The kernel determines what exists, what an individual knows, what actions are possible, and what actually happens. The LLM occasionally helps a notable choose what to pursue and how to negotiate it.**

The research supports this direction. Generative Agents demonstrated believable small-group coordination; Project Sid demonstrated larger societies using separated cognitive and execution components; AgentSociety demonstrated substantial infrastructure scale. None of these results establishes that continuously prompting hundreds of individuals will support centuries of simulation while sharing a 12 GB GPU with a demanding renderer. Their workloads, time horizons, and performance measurements are materially different from TCE’s. [arXiv](https://arxiv.org/html/2304.03442)

My recommended endpoint is **100–300 persistent notable profiles, one shared model, very few simultaneous inference requests, event-triggered deliberation, and an always-available rule-based fallback**. Start with an external inference process; consider deeper GPU scheduling integration only after the behavior proves valuable.

---

## 1. Options: what to put under LLM control

The following are engineering options, not a ranking of model intelligence.

| Architecture | How it works | Principal trade-off | Fit for TCE |
| --- | --- | --- | --- |
| **Rules decide; LLM expresses** | Utility AI or a planner selects actions. The LLM produces dialogue, letters, summaries, or explanations grounded in those decisions. | Little additional behavioral emergence, but strong control over correctness and cost. | **Best first implementation.** |
| **LLM selects bounded intentions** | The kernel supplies knowledge, goals, commitments, and feasible alternatives. The LLM selects a plan; ordinary AI executes it. | Adds context-sensitive decisions without putting inference on the simulation’s critical path. | **Recommended long-term design.** |
| **Full generative agent** | Perception, memory retrieval, reflection, planning, conversation, and replanning repeatedly invoke models. | Rich behavior, but many calls and several ways for interpretation errors to feed back into state. | Research mode or a handful of showcase characters. |
| **One model call runs a council or scene** | A prompt contains several characters and generates their joint interaction or decisions. | Lower call count, but encourages shared knowledge, correlated personalities, and narrator-controlled outcomes. | Suitable for presentation; risky for independently acting political opponents. |
| **Offline teacher, cheap runtime policy** | A larger model generates training examples, candidate plans, or authored content. Reviewed outputs train a small policy or become reusable rules. | Runtime is inexpensive, but coverage and validation move into development. | Valuable later, especially for fast-forward. |

The full-agent pattern comes directly from Generative Agents’ memory–reflection–planning architecture. The hybrid pattern has stronger execution-oriented precedents: Project Sid separates concurrent cognitive and action systems, while CICERO couples strategic planning to a controlled dialogue model rather than expecting language generation alone to supply strategy. [arXiv](https://arxiv.org/html/2304.03442)

For TCE, reserve deliberation for decisions such as **sponsoring a communal granary, changing a faction’s negotiating position, proposing a law, selecting a research objective, or commissioning a building programme**. Eating, travelling, working, allocating already-committed labour, and responding to immediate danger should not need inference.

An LLM can select a meaningful objective without individually planning every footstep needed to achieve it.

---

## 2. Trade-offs: scale, latency, cost, and a shared 12 GB GPU

### 2.1 “Number of agents” is not a throughput measurement

The most useful published performance comparison is AgentSociety’s separation of environment execution from LLM-driven interaction:

| Published workload | Measurement | What it establishes |
| --- | --- | --- |
| AgentSociety environment, 10,000 individuals | **9.129 ms mean environment step** | Its conventional environment simulator is relatively inexpensive. |
| AgentSociety, 10,000 LLM agents, 32 groups | **458.82 seconds per interaction round** | Large populations are orchestratable, but deliberation is much slower than environment stepping. |
| Same LLM configuration | **8.05 seconds mean LLM call**, approximately **430 input / 76 output tokens per call** | Even compact prompts can have substantial service latency. |

These experiments used a **64-core cloud machine and external DeepSeek-V3 APIs**, with API tests scheduled during off-peak hours. The round is not a rendering frame or a simulated day, and the environment-only timing excludes the LLM workload. These are 2025 measurements, not predictions for newer models or your PC. [arXiv](https://arxiv.org/html/2502.08691v1)

The lesson is architectural: **fast simulation infrastructure does not make frequent model decisions inexpensive**.

### 2.2 Budget calls against both simulated time and real time

Let:

* \(N\) be the number of eligible notables;
* \(f\) be decisions per notable per simulated day;
* \(s\) be simulated days advanced per real second.

Then the required request rate is:

\[
\lambda=Nfs
\]

For **300 notables**, each receiving one decision per simulated day:

| Simulation speed | Required calls | At 150 output tokens per call |
| --- | --- | --- |
| One simulated day per real minute | 5 calls/second | 750 output tokens/second |
| One simulated day per real second | 300 calls/second | 45,000 output tokens/second |

Those are arithmetic requirements, not measured capabilities. They exclude input processing, retries, reflections, and conversations.

Conversely, a global limit of **10 calls per real minute** takes **30 real minutes** to give each of 300 notables one turn. Thus, “a few hundred LLM-enabled people” must mean **a few hundred eligible persistent identities**, not a few hundred continuously thinking sessions.

A useful latency model is:

\[
L \approx L\_{\text{queue}}+
\frac{T\_{\text{uncached input}}}{R\_{\text{prefill}}}+
\frac{T\_{\text{output}}}{R\_{\text{decode}}}
\]

For illustration, at a hypothetical 50 generated tokens/second, a 150-token response takes three seconds **before** input processing and queueing. This is not a 4070 Ti benchmark.

### 2.3 Cloud cost compounds across history

As a concrete pricing reference, Anthropic’s published standard rates for **Claude Haiku 4.5**, checked for this report, are **$1 per million input tokens and $5 per million output tokens**. This is a costing example, not a claim that Haiku is the best TCE model. [Claude Platform](https://platform.claude.com/docs/en/about-claude/pricing)

Assume **1,500 total billed input tokens and 150 output tokens per decision**, with no cache discounts:

\[
C\_{\text{decision}}=
\frac{1500(1)+150(5)}{10^6}
=\$0.00225
\]

| Deliberation policy | Calculated cost |
| --- | --- |
| 300 notables, once per simulated day | **$246.38 per simulated year** |
| Same policy for a century | **$24,637.50** |
| 300 notables, once per seven simulated days | **About $35.20 per simulated year** |
| Global cap of 10 decisions per real minute | **$1.35 per real hour** |

These calculations assume 365-day years. Additional calls multiply the cost. The 1,500-token assumption must include instructions, schemas, memories, and any provider-added tool overhead; it is not merely the visible situation description.

For an endless simulation, **a wall-clock spending ceiling is more controllable than a promise to deliberate every simulated day**. It also creates a fidelity issue at different speeds, discussed in Section 4.

### 2.4 Local inference: fitting the weights is only the first test

The GPU budget is:

\[
V\_{\text{inference}} =
V\_{\text{weights}}+
V\_{\text{KV or recurrent state}}+
V\_{\text{work buffers}}+
V\_{\text{runtime overhead}}
\]

The theoretical packed-weight minimum for a four-bit model is roughly:

\[
V\_{\text{weights,min}}=P \times 0.5\text{ bytes}
\]

Thus, four billion parameters require approximately **2 GB of packed weights alone**, and eight billion require approximately **4 GB**. Real quantized formats add metadata and often retain some tensors at higher precision.

For an illustrative allocation—not a measurement of TCE—if rendering consumes 8 GB and you reserve 1 GB of headroom, only 3 GB remains. A nominally small model can then become tight once its context and work buffers are included.

More importantly, **VRAM headroom does not imply GPU execution headroom**. NVIDIA’s NVIGI exists specifically to coordinate inference and graphics workloads, including access to the game’s D3D12 device and command queue. A separate CUDA process does not automatically gain that coordination. [NVIDIA Developer](https://developer.nvidia.com/blog/bring-nvidia-ace-ai-characters-to-games-with-the-new-in-game-inference-sdk/)

For your hardware, I would evaluate a quantized **1–4B-class model**, short contexts, and one active request before considering anything larger. Do not start by allocating an 8B model merely because it loads.

Useful comparison candidates include:

* **Qwen3-4B-Instruct-2507:** a specifically non-thinking instruction model, suitable as a pinned baseline.
* **Qwen3.5-4B:** a 2026 challenger with a different architecture; its model card says thinking is enabled by default and documents an explicit non-thinking configuration. Evaluate that mode for bounded decisions rather than allowing long reasoning output. [Hugging Face](https://huggingface.co/Qwen/Qwen3-4B-Instruct-2507)

Neither model card supplies a benchmark for **TCE + UE 5.8 + 1440p rendering on your 4070 Ti**. Quantization quality, backend support, context length, and simultaneous rendering all require measurement.

### 2.5 Deployment choices

**Local CPU inference** avoids GPU allocation, but competes with the Rust kernel for CPU time and memory bandwidth. Treat its thread count as a budget, not “use every core.”

**Local GPU inference** offers an offline path, but must be admitted according to frame-time and memory headroom. Making the request asynchronous prevents a blocking call; it does not remove resource contention.

**Cloud inference** avoids local inference contention, but adds costs, service variability, connectivity requirements, and an external dependency. It is useful as an optional high-quality provider or development reference, not something the world must wait for to function.

---

## 3. Precedents: what the projects actually learned

### 3.1 Generative Agents — the foundational architecture

Park and colleagues’ 2023 system combined a natural-language memory stream, retrieval by relevance/recency/importance, reflection, and hierarchical planning. Its Smallville demonstration used **25 agents over two simulated days**. Coordination around a seeded Valentine’s party showed that information and intentions could propagate without scripting each participant’s actions. [arXiv](https://arxiv.org/html/2304.03442)

Its limitations are equally relevant: misunderstood environmental constraints, invented or embellished memories, and excessively polite or cooperative behaviour. The paper reports **thousands of dollars in token credits and multiple wall-clock days** for the demonstrated run; that is historical cost, not a current-model estimate. [arXiv](https://arxiv.org/html/2304.03442)

**TCE lesson:** borrow episodic memory and selective reflection, but store the authoritative facts outside their natural-language summaries. Short-term believability is not evidence of multi-century consistency.

### 3.2 Project Sid — orchestration matters as much as the model

Project Sid’s 2024 report describes **50–100 agents within individual societies and 500–1,000 across interacting societies** in Minecraft. Its PIANO architecture combines parallel cognitive and non-cognitive components through shared agent state, with mechanisms for coherent decisions, action awareness, and social interaction. Reported outcomes include differentiated roles, collective rule changes, and cultural transmission. [arXiv](https://arxiv.org/html/2411.00114v1)

The important engineering contribution is the separation between slower cognition and continuing execution. The system also addresses practical problems such as excessive conversation and discrepancies between intended and actual actions.

**TCE lesson:** an agent needs to know whether its plan succeeded. “I arranged food deliveries” must not become a memory merely because the model generated that sentence. Minecraft demonstrations remain bounded by the environment’s authored mechanics; they do not establish unconstrained historical development. [arXiv](https://arxiv.org/html/2411.00114v1)

### 3.3 AgentSociety — substantial infrastructure, not desktop-speed cognition

AgentSociety’s 2025 architecture combines needs, cognition, planning, and behaviour with urban, social, and economic environments. Its asynchronous execution infrastructure is the main scale precedent; the performance distinction in Section 2 is its most important implication for TCE. The social experiments are evidence within their tested settings, not a general validation of human societies. [arXiv](https://arxiv.org/html/2502.08691v1)

**TCE lesson:** keep expensive decisions outside the environmental machinery. Do not import a distributed research stack merely to serve a few hundred local notables.

### 3.4 AgentSociety 2 — a materially different 2026 system

The current project has moved beyond the original architecture. AgentSociety 2’s documentation, labelled **v2.9.3** when reviewed, represents agents as persisted workspace state and reconstructs lightweight agents in batched Ray tasks. This avoids treating every personality as a permanently running heavyweight process. [AgentSociety 2](https://agentsociety2.readthedocs.io/en/latest/architecture.html)

The July 2026 paper also provides a useful warning about environment integration. On its multi-step routing benchmark, **GLM-4.7 achieved 57.8% strict success with CodeGenRouter versus 29.4% with ReAct**. Strict success requires the correct function sequence and parameters; this is not a general invalid-action rate, but it shows that successful API orchestration remains a separate problem. [arXiv](https://arxiv.org/html/2607.11895v2)

**TCE lesson:** borrow state/task separation and reuse mechanisms. Do not execute model-generated general-purpose code inside the simulation.

### 3.5 Other 2024–2026 results worth carrying forward

| Work | Result or architecture | Relevant boundary |
| --- | --- | --- |
| **OASIS**, 2024–2025 | Social-platform simulation scaling to one million agents, with recommendation systems and actions such as posting, commenting, and following. | A million social-media identities is not a million physically simulated inhabitants making frequent decisions. [arXiv](https://arxiv.org/abs/2411.11581) |
| **AIvilization v0**, February 2026 | Hierarchical planning, short/long-term memory, adaptive profiles, an action simulator, and tiered repair in a resource-constrained economy. Reports tens of thousands of cumulative agents and over 600,000 transactions. | Cumulative participation is not simultaneous throughput. Human steering and authored economic mechanisms matter when interpreting emergence. [arXiv](https://arxiv.org/html/2602.10429v1) |
| **Park et al., revised June 2026** | Agents grounded in interviews, surveys, or both for 1,052 people. Held-out survey accuracy reached 83%, 82%, and 86% of participants’ own test–retest consistency, respectively. | These are relative-to-human-repeatability scores, not “86% accurate human simulation.” Modern survey prediction does not establish ancient-population realism. [arXiv](https://arxiv.org/abs/2411.10109) |
| **Concordia** | Modular generative agents interact through a Game Master that adjudicates outcomes. | Useful separation of action proposal and adjudication; TCE should retain its Rust kernel as adjudicator rather than letting prose establish physics. [GitHub](https://github.com/google-deepmind/concordia) |
| **CICERO** | A strategic planning engine controls a dialogue model in Diplomacy. | Strong precedent for language grounded in plans, but a bounded game rather than an open-ended society. [Meta AI](https://ai.meta.com/research/cicero/) |

### 3.6 Personality and identity are not solved by a biography prompt

A 2025 study of over 2,000 agent-to-agent conversations across 60 configurations found **role “echoing” in 5–70% of conversations**, depending on configuration: agents increasingly adopted aspects of their interlocutor’s identity or goals. More reasoning did not reliably eliminate it. [arXiv](https://arxiv.org/html/2511.09710v1)

A June 2026 personality-anchoring study found more stable, behaviourally consequential differences in its tested conversations. That is encouraging, but it validates a particular anchoring method over particular interactions—not lifelong consistency. [arXiv](https://arxiv.org/html/2606.06936v1)

For TCE, personality must therefore be observable in **choices, loyalties, risk-taking, and commitment**, not only distinctive phrasing.

### 3.7 Shipped and publicly playable precedents

**inZOI’s Smart Zoi is particularly relevant.** KRAFTON’s March 26, 2025 Early Access guidance described it as experimental and documented concrete limits: journal creation and schedule editing were disabled at **5× speed or higher**, and a particular situation involving several sleepy Zois could cause lag. Those are historical launch-period issues, not claims about the current build. They demonstrate why accelerated simulation and inference need explicit coordination. [inZOI](https://playinzoi.com/en/news/8419)

**GoodAI’s AI People** released a public alpha in September 2024 with cloud-funded AI interactions. Its launch announcement explicitly connected its subscription/credit model to inference infrastructure costs and described local inference as planned work at that time. [GoodAI](https://www.goodai.com/ai-people-alpha-officially-released/)

**Suck Up!** is a commercial example of real-time, personality-driven AI conversation. It is evidence that bounded AI interaction can be a game’s central mechanic, not a precedent for population-scale autonomous history. [Steam Store](https://store.steampowered.com/app/2726370/Suck_Up/)

I found no documented result in these sources combining TCE’s population, centuries-long horizon, hundreds of independently deliberating notables, and your specific rendering target.

---

## 4. Recommended TCE design

The following is a proposed architecture, not a claim that an existing framework already implements it.

### 4.1 Keep the authoritative boundary narrow

```
Rust simulation
    → bounded perception + goals + feasible choices
    → bounded request queue
    → inference broker / pluggable provider
    → untrusted proposal
    → kernel validation at a decision boundary
    → committed intention
    → ordinary AI execution
    → observed outcome recorded by the kernel
```

The provider interface should support **rule-only, recorded replay, local CPU, local GPU, and optional remote inference**. Provider failure must not make a notable stop being a person.

The kernel should retain ownership of identity, households, employment, property, relationships, beliefs, institutional authority, action costs, and consequences. An accepted LLM proposal changes an intention; it does not bypass those systems.

For the DLL boundary, use owned request/response buffers and a versioned C-compatible interface. Do not expose borrowed Rust world state to an inference callback. Shutdown must cancel or drain inference work before unloading the kernel or its plugin.

### 4.2 Start with an external inference process

For a solo developer, my first implementation would be a pinned **llama.cpp server** launched as a local sidecar. It already supplies request serving, parallel slots, continuous batching, prefix reuse, and constrained-output support. This avoids building a model-serving system inside your Rust DLL. [GitHub](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)

Keep the endpoint local and authenticated, disable unnecessary capabilities, and never expose arbitrary file access or code execution as agent tools.

Process separation improves fault isolation and packaging flexibility. It **does not isolate GPU resources**.

Once profiling justifies deeper integration, evaluate **NVIGI 1.7.0** through the UE C++ plugin. Its compute-in-graphics path is directly relevant, but the public Unreal sample currently identifies itself as targeting **UE 5.6**, not 5.8. Treat adaptation and packaged-build validation as explicit work. [GitHub](https://github.com/NVIDIA-RTX/NVIGI)

Do not adopt Ray, MQTT, a distributed vector database, and a multi-agent orchestration framework merely because a research project uses them.

### 4.3 Give the model choices, not unrestricted control

Initially, let the kernel generate a small candidate set:

```
Choice 12: Retain the existing grain levy.
Choice 13: Propose a temporary levy reduction.
Choice 14: Request a council meeting about emergency grain purchases.
Choice 15: Delay reconsideration until the harvest estimate arrives.
```

The output can be very small:

```
{
  "choice_id": 14,
  "reason_code": "food_security",
  "evidence_ids": [381, 407]
}
```

The request envelope—not model-generated text—carries actor ID, request ID, relevant state revision, and deadline.

Later, add parameterized actions such as a bounded offer, policy proposal, or building commission. Their types should come from TCE’s authored primitives.

**Grammar-constrained decoding is only the first validation layer.** llama.cpp can constrain token generation through grammars and supported JSON-schema conversions; this does not establish that a target exists or an action is possible in the current world. [GitHub](https://github.com/ggml-org/llama.cpp/blob/master/grammars/README.md)

The kernel must then validate entity liveness, knowledge, authority, resources, preconditions, parameter ranges, and conflicts. Apply accepted changes atomically.

A crucial distinction for TCE: **physically impossible is not the same as illegal in-world**. Theft, rebellion, and treaty-breaking can be valid simulated actions with consequences. Conversely, a person who may *propose* a law must not gain the power to enact it merely by returning an `EnactLaw` instruction.

### 4.4 Make delayed answers safe

Every request should have a bounded useful lifetime and one of four outcomes: **accepted, rejected, expired, or superseded**.

Allow at most one pending deliberation per notable. Coalesce repeated triggers. Revalidate against relevant state when the answer arrives; a global tick number is too aggressive because unrelated world activity would invalidate everything.

Reserve scarce resources at commitment or execution, not when constructing the prompt. A plan may be valid when selected and fail later; the executor must report that failure.

Use a cheap fallback rather than an unbounded repair conversation. For valuable decisions, permit one bounded repair attempt; malformed, refused, truncated, stale, and unavailable responses all need defined handling.

Most importantly, record **actual outcome events**. Never record intended outcomes as accomplishments.

### 4.5 Put identity and memory in structured state

A notable should have several distinct layers:

| Layer | Owner and update rule |
| --- | --- |
| **Identity and slow traits** | Kernel-owned; change only through explicit, bounded development rules. |
| **Current needs and commitments** | Simulation state; revised by actual events and execution. |
| **Knowledge and beliefs** | Agent-specific facts, uncertainty, hearsay, and provenance. |
| **Episodic memories** | References to observed events, with salience and decay. |
| **Narrative summaries** | Replaceable derived text; never authoritative history. |

Generate a compact prompt from these layers rather than maintaining a lifelong chat transcript.

Use heterogeneous private incentives, formative events, relationships, and behavioural examples. Do not expect “stubborn, ambitious, kind” to produce adequately differentiated policy choices.

At succession, institutional records and inheritable property may pass on; private memories should not automatically become the heir’s knowledge. Centuries of simulation require explicit forgetting, preservation, and transmission.

The same boundary prevents technological leakage. A model can suggest a modern institution or machine because of its training; TCE should accept only proposals expressible through currently available capabilities and authored mechanisms. **Naming a technology is not discovering it.**

### 4.6 Bound deliberation, not just output length

My proposed starting limits are:

| Control | Initial experimental setting |
| --- | --- |
| Eligible notables | 100–300 |
| Concurrent local requests | 1; test 2 only after profiling |
| Typical input | Approximately 1,000–2,000 tokens |
| Context ceiling | Initially around 4,000 tokens |
| Typical output cap | 100–200 tokens |
| Global admission | Initially 5–10 requests per real minute |
| Triggers | Material goal conflict, failed commitment, major new information, institutional decision |
| Ordinary activity | Always handled without an LLM |

These are **starting limits, not promised sustainable performance**.

Combine per-agent cooldowns with a global token budget and queue cap. Give requests an impact estimate, but preserve fairness across settlements and factions. Otherwise the most visible or frequently inspected characters may systematically receive more intelligence.

A famine should not provoke thousands of simultaneous requests. Immediate responses should come from ordinary AI; deliberation can later address exceptional policy choices.

### 4.7 Cache four different things

**Existing plans.** The cheapest request is the one avoided because the current commitment still applies. Maintain dependency checks and reconsider only when relevant conditions change.

**Prompt prefixes.** Put shared rules and schemas before agent-specific and rapidly changing content. llama.cpp’s prefix reuse can avoid reprocessing matching tokens, but does not eliminate generation or make every character’s state interchangeable. [GitHub](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)

**Exact decisions.** Reuse only when model version, template, schema, actor profile, knowledge, relevant state, and sampling policy match. Broad semantic caching of “similar people in similar situations” can erase the very differences TCE is trying to preserve.

**Reusable plan structures.** Cache validated negotiation or construction-plan skeletons independently from personal choices. Revalidate bindings and costs each time.

Batch independent requests through the inference backend, not by placing an entire population into one prompt. Begin with small batches: throughput gains must be weighed against memory use and response latency. Cloud batch-processing services are useful for offline evaluation or content generation, not a substitute for timely live decisions.

### 4.8 Treat fast-forward as a model-semantics decision

**Reducing LLM calls at high speed changes the decision process. It is not automatically an equivalent faster simulation.**

Choose and document one of these policies:

* **Expressive mode:** rules make all consequential decisions; inference changes presentation only. Simulation outcomes need not depend on inference availability.
* **Live deliberative mode:** inference may change intentions, but expired requests fall back. Hardware and simulation speed can affect which proposals are available.
* **Controlled/offline mode:** simulation waits at designated decision boundaries, or consumes precomputed decisions, preserving the specified deliberation schedule at the cost of throughput.

For normal TCE play, I recommend live deliberation as an explicitly optional mode, with persistent plans spanning many ordinary updates. For comparisons and debugging, provide recorded decision playback.

Save provider/model fingerprints, accepted proposals, fallbacks, and their commit ticks. Temperature zero alone is not a reproducibility guarantee; llama.cpp itself notes possible backend/batch-dependent differences when reusing prompt caches. [GitHub](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)

### 4.9 Failure modes and acceptance tests

| Failure | TCE safeguard | Test |
| --- | --- | --- |
| Invalid or invented action | Candidate-conditioned output plus kernel validation | Fuzz IDs, ranges, missing fields, and stale entities; reject without mutation. |
| Personality homogenization | Persistent private incentives and behavioural anchors | Compare decisions—not prose—across contrasting profiles on the same situations. |
| Identity drift | Fresh bounded contexts; structured identity | Long interaction sequences must not silently replace goals, affiliations, or relationships. |
| Fabricated memories | Event provenance; summaries cannot create facts | A false summary must not create property, debt, knowledge, or past accomplishments. |
| Endless discussion or replanning | Round limits, cooldowns, commitment duration | Adversarial negotiations must terminate or fall back within budget. |
| Modern-knowledge leakage | Capability and knowledge gates | Reject unsupported technology and institutional powers. |
| Prompt injection through dialogue or mods | Treat embedded text as data; no external tools | In-world “instructions” must not alter the protocol or grant capabilities. |
| Frame-time regression | Admission control and bounded GPU work | Test cold/warm inference during the busiest visible city scene. |

Add a small decision-regression suite: fixed snapshots of famine, succession, debt, building disputes, trade offers, and competing loyalties. Evaluate validity, response latency, commitment consistency, and meaningful sensitivity to changed circumstances.

For performance, measure **p95/p99 frame time**, simulation throughput, peak VRAM, queue age, expiry rate, and uncached input/output tokens. Do not accept a satisfactory average frame rate while inference causes periodic long frames.

For a solo developer, the implementation sequence should be: **rule-only interface and replay first; non-causal dialogue second; one bounded consequential decision type third; GPU integration last**. Each stage remains useful even if the next is deferred.

---

## 5. Sources and version guide

The links below identify the most useful implementation and research entry points. Rolling repositories should be pinned to an exact commit before integration.

| Resource | Version/date relevant here | Why read it |
| --- | --- | --- |
| [Generative Agents paper](https://arxiv.org/abs/2304.03442?utm_source=chatgpt.com) · [code](https://github.com/joonspk-research/generative_agents?utm_source=chatgpt.com) | 2023 paper; repository is evolving | Memory, reflection, planning, and early failure analysis. |
| [Project Sid](https://arxiv.org/html/2411.00114v1?utm_source=chatgpt.com) | October 2024, v1 | Parallel cognition, execution awareness, and society-scale experiments. |
| [AgentSociety paper](https://arxiv.org/html/2502.08691v1?utm_source=chatgpt.com) | February 2025, v1 | The performance measurements quoted above. |
| [AgentSociety repository](https://github.com/tsinghua-fib-lab/AgentSociety) · [architecture documentation](https://agentsociety2.readthedocs.io/en/latest/architecture.html?utm_source=chatgpt.com) | Current documentation labelled v2.9.3 when reviewed | Current implementation; do not confuse it with the original paper’s runtime. |
| [AgentSociety 2 paper](https://arxiv.org/html/2607.11895v2?utm_source=chatgpt.com) | July 2026, v2 | Batched execution, reusable routing, and integration benchmarks. |
| [AIvilization v0](https://arxiv.org/html/2602.10429v1?utm_source=chatgpt.com) · [OASIS](https://arxiv.org/abs/2411.11581?utm_source=chatgpt.com) | February 2026; OASIS v5 March 2025 | Economic-agent architecture and large social-network simulation. |
| [Grounded individual simulations](https://arxiv.org/abs/2411.10109v3) | June 28, 2026, v3 | Updated results; the earlier widely quoted 85% figure is not the revised abstract’s breakdown. |
| [Role-echoing study](https://arxiv.org/html/2511.09710v1?utm_source=chatgpt.com) · [personality anchoring](https://arxiv.org/html/2606.06936v1?utm_source=chatgpt.com) | November 2025; June 2026 | Identity failure and bounded evidence for improved persona consistency. |
| [Concordia code](https://github.com/google-deepmind/concordia?utm_source=chatgpt.com) · [v2.0 tutorial talk](https://www.youtube.com/watch?v=2FO5g65mu2I) | Current repository; tutorial explicitly covers v2.0 | Modular agent construction and adjudication. |
| [llama.cpp server](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md?utm_source=chatgpt.com) · [grammars](https://github.com/ggml-org/llama.cpp/blob/master/grammars/README.md?utm_source=chatgpt.com) | Rolling documentation checked September 2026 | Serving, batching, slots, prefix caching, and constrained generation. |
| [NVIGI source](https://github.com/NVIDIA-RTX/NVIGI?utm_source=chatgpt.com) · [Unreal sample](https://github.com/NVIDIA-RTX/NVIGI-UEPlugin) · [integration guide](https://developer.nvidia.com/blog/bring-nvidia-ace-ai-characters-to-games-with-the-new-in-game-inference-sdk/?utm_source=chatgpt.com) | SDK 1.7.0; UE sample labelled 5.6; guide February 2025 | Windows inference and graphics coordination; verify the UE 5.8 port separately. |
| [Qwen3-4B-Instruct-2507](https://huggingface.co/Qwen/Qwen3-4B-Instruct-2507?utm_source=chatgpt.com) · [Qwen3.5-4B](https://huggingface.co/Qwen/Qwen3.5-4B?utm_source=chatgpt.com) | July 2025 baseline; 2026 challenger | Candidate models, architecture, and non-thinking configuration. |
| [Claude API pricing](https://platform.claude.com/docs/en/about-claude/pricing?utm_source=chatgpt.com) | Checked September 27, 2026 | Rates used for the worked cost example; not a model-selection recommendation. |

**Bottom line:** TCE should gain richer interpretation and negotiation from LLMs without making personal continuity, physical causality, or simulation progress depend on them. The decisive architecture is not “an LLM for every notable”; it is **persistent people, bounded proposals, infrequent deliberation, and consequences enforced by the kernel**.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927c8-15a4-83ea-8141-160d62f60232)
