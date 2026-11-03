# ADR-0039: A model deliberator, and more than one viewer

Status: Accepted (2026-10-30, by Claude under the owner's standing authority)
Date: 2026-10-30
Milestone: M12
Amends: ADR-0013 §5 (a second `Deliberator`), ADR-0001 (viewers beyond this machine, and roles)

## Context

Plan §7's M12 is "Optional LLM `Deliberator`, multi-viewer, close-up fidelity (MetaHuman-grade
notables)"; plan §8's open question 2 asks which LLM backend, which model and what cost limits.
Research 01-13 recommends the model as an optional, asynchronous proposer of bounded choices with
an always-available rule fallback, an external local inference process, and recorded replay
(01-13 Executive recommendation, §4). The brief is `docs/briefs/m12-deliberator-viewers.md`.
Close-up fidelity is Unreal work and, like M2's client, needs the owner's Windows PC.

## Decision

### 1. Parts

M12a the deliberator (§2–§5); M12b viewers (§6); M12c their demo. Close-up fidelity waits with
M2's Unreal client for the owner's PC (§8).

### 2. The model chooses among the kernel's moves

A `ModelDeliberator` implements `Deliberator` beside `RuleDeliberator`. At the polity review the
kernel lists a notable's feasible moves as now; the model is shown who the notable is, what they
know that the review already reads (their household's forecasts, those who regard them, the
support they expect) and the moves by number, and answers with one number or none. An answer that
names no listed move is refused; the rule deliberator then chooses, as it does for every request
the model does not answer. The model never enacts, spends or creates anything: what follows a move
is what follows it now.

### 3. Providers, and the backend (plan §8, open question 2)

- **Rule-only** (the default): no model; every world lives exactly as now.
- **HTTP**: an OpenAI-compatible chat-completions endpoint. The default is local: a llama.cpp
  server on this machine (01-13 §4.2), any model the owner loads; a remote endpoint only where the
  owner names one, its key read from an environment variable, never stored in the repository, a
  save or a log.
- **Recorded**: answers read back from the save, for replay.
- **Cost limits**: at most one request in flight, a wall-clock deadline per request (a design
  prior of 10 s), and a budget of requests per real hour (60) the owner may change; past either,
  rules decide.

### 4. Answers are saved

Each accepted answer is saved with the notable, the day, the moves offered (by a digest) and the
provider's fingerprint (endpoint kind and model name). A world loaded with the recorded provider
reads them back, so it relives its choices exactly; a request with no saved answer falls back to
rules. The receipt shows who chose: "by the model" or "by rule (the model did not answer)".

### 5. In-world text is data

Names, laws and words in a prompt are quoted as data; the reply is parsed for a single number and
nothing else; no tool, file or network access is offered to the model (01-13 §4.9).

### 6. Viewers

Off by default the socket stays on 127.0.0.1 as now. The owner may open it to the local network at
launch (`--viewers lan`), which requires a session token. Each connection has a role: the first
local connection drives (commands, time); every other connection watches (snapshots, events,
queries) and its commands are refused by name, until the driver hands over. The observer shows its
role and who drives.

### 7. The boundary

Saved answers are a new saved section (saves bump); the wire gains the role and the hand-over (wire
bumps); content is unchanged.

### 8. Deferred

Close-up fidelity (MetaHuman-grade notables) and every Unreal-side part wait for the owner's PC.
Non-causal model text (a notable's letters, explanations) and any model use beyond the polity
review wait for the first part to prove valuable (01-13 §4.9's sequence).

## Consequences

- A world run with the rule provider is unchanged; one run with a model differs run to run unless
  replayed from its save.
- The owner chooses whether to install a model; nothing in the build needs one, and the chain
  tests the provider against a stub.
- Opening the socket to the network is an explicit launch choice with a token.

## Alternatives considered

- **The model as the simulation** (01-13 §1, full generative agents): many calls, state errors
  fed back; rejected.
- **A remote API by default**: cost and data leave the machine; local first, remote by choice.
- **Spectators by a separate read-only server**: duplicates the socket; roles on one socket are
  enough.

## Revisit when

- The model's choices prove valuable enough to extend past the polity review.
- Unreal runs on the owner's PC: close-up fidelity.
- Viewers beyond the local network are wanted.
