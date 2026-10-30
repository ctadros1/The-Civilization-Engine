# M12 design brief: an optional model deliberator, and more than one viewer

**Scope.** Plan §7's M12 is "Optional LLM `Deliberator`, multi-viewer, close-up fidelity
(MetaHuman-grade notables)", and plan §8's open question 2 asks "local vs API, which model, and
cost limits". Close-up fidelity is Unreal work, like M2's: it needs Unreal on the owner's Windows
PC, which cloud sessions do not have (plan §9, "M3 goes ahead of M2's Unreal work"). This brief
covers the deliberator and the viewers. Written 2026-10-30 by Claude under the owner's standing
authority.

**Read:** 01-13 (LLM-driven agents) in full, with §4 read closely. Citations are to its sections.

## What the code already has

- **The `Deliberator` trait** (`civ-agents/src/polity.rs`, ADR-0013 §5): a notable choosing among
  institutional moves (`MoveOption`, with the forecasts and support behind each) or none, with a
  receipt of scores. One rule-based implementation (`RuleDeliberator`, a softmax), called at one
  place, the polity review (`population/polity.rs`), sequentially.
- **The observer socket** (`civ-host/src/server.rs`, ADR-0001): HTTP and a WebSocket on 127.0.0.1
  only, refusing pages from other origins, with an optional session token. Several connections
  already share the engine's snapshots and events; each may command.

## 1. The deliberator (01-13)

1. **The model chooses among the kernel's moves; it never acts** (01-13 §1, §4.1, §4.3): the
   kernel lists the feasible moves and validates the answer; an answer naming no listed move, or
   one no longer feasible, is refused. Proposing a law never enacts it.
2. **Event-triggered and few** (Executive recommendation; §2.2): 100–300 notables, very few
   requests at once. TCE's reviews are already rare (a polity review, yearly for most people).
3. **Always a fallback** (§4.4): every request has a deadline and one of four ends: accepted,
   rejected, expired, superseded. Rules decide whatever the model does not.
4. **An external process first** (§4.2): a local llama.cpp server as a sidecar, kept local, no
   tools, no file access.
5. **Fast-forward is a semantic choice** (§4.8): say which mode a world runs in; save the
   provider's fingerprint and every accepted answer, so a run can be replayed.
6. **Treat in-world text as data** (§4.9): prompt injection through names or laws must not
   change the protocol.

**My proposal.** The kernel keeps its trait and gains a second implementation that consults a
`Provider`: rule-only (today), recorded (answers read back from the save), or HTTP (an
OpenAI-compatible chat endpoint, the llama.cpp server's). The host owns the provider; the kernel
sees only a choice. Off by default: a world without a model lives exactly as now. Answers are
saved, so a reload or replay reads them back instead of asking again.

## 2. Viewers

More than one person watching one world (plan §2) needs: viewers on other machines, which the
socket now refuses; a way to keep a spectator from driving; and nothing that weakens the local
default. **My proposal:** off by default; the owner may open the socket to the local network with a
required token; each connection is a driver or a spectator (the first local one drives; others
watch and may query, not command, unless the driver hands over).

## 3. Guards

- No model, no change: every digest equal with the rule provider.
- No credentials in the repository or the save: an endpoint's key comes from the environment.
- No new network exposure unless the owner asks for it at launch.
