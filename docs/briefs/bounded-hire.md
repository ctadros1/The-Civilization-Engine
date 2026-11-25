# Brief: a seeker of work weighs the offers within reach of what it knows

Status: **proposed, for the owner's review before any build** (2026-11-25). Plan §8 24, seventh
item (the 100k bench workload). One of three ways the owner was offered to close what four exact
cuts left (bench run 127: the 100k workload 1.56× M9 against the clause's 1.10×).

## 1. What happens now

Each time a person weighs work for hire (`best_job`, `population/firm.rs`), every employer
household of its settlement, and those of other settlements with a field within the work's walk
(`hire_scan_m`), works out its offer afresh: reaping, threshing, or preparing and sowing
(`reap_offer_with`, `population/reapers.rs`), from all its fields, its hands and its grain. In a
settlement of 10,000 that is some 2,000 employers on every decision. In the measured 100k day this
is about a third of the main thread after the four cuts, and it grows with the square of a
settlement's size. It is also more than the research asks.

## 2. What the research asks (08-10 §5.3, "Use bounded local matching")

> 2. Workers inspect a small set of reachable offers through local and social networks.

with EURACE-family models as precedent ("workers search subsets of offers"). A day labourer of the
periods the engine models learned of work from kin, neighbours and those met, and at the places
work was given out, not from a census of every farm.

## 3. Proposed design

A seeker weighs the offers of **the employers it knows of today**, in this order, and no others:

1. its people's close kin's households and those its people hold a tie with (the same lists help
   with mending reads, `mend_candidates`), and
2. the households of the fields nearest its home within the work's walk, at most `hire_seen`
   of them (content, a tuning value: the fields one passes or can see from one's own ground),
   found by the field grid already kept (`fields_near`),

each employer once, in a fixed order (kin by distance, ties by regard, then fields by distance,
ties broken by id), so the result is a function of saved state alone (ADR-0011 §4).

What does not change: how an offer is worked out, wages and their bidding up, who may work for
hire, the observer's words. An employer no seeker knows of goes short of hands, as one far from
the village's paths would, and its own household works or the crop waits; the market's later
signals (wages bid up when the harvest is short of hands, `reap_wage`) still answer it.

## 4. Cost

The offers worked out per decision fall from every employer of the settlement to at most the
kin and ties plus `hire_seen`; with `hire_seen` near 20 the hiring share of the 100k day should
fall from a third to a few per cent, which would bring the workload near the clause. Not measured:
to be shown by a bench before it is proposed for promotion.

## 5. Validation

- Scenes (civ-sim/tests): a seeker takes the best offer among those it knows, not a better one it
  does not; kin are weighed though their fields are past `hire_seen`; an employer known to nobody
  goes unhired and its wage rises after (existing bidding up).
- The hiring and harvest scenes now passing stay passing, or each change is named.
- A long run: the dashboard's Harvest margins row (sessions of reaping hired, gleaning) and
  Population, against long runs 123 and 126; fewer hired sessions is expected and reported, not
  graded.
- The bench on node3 for the 100k clause; `NUDGE:` entries for `hire_seen`.

## 6. For the owner

- Accept this, or change it, before it is built. It is a behaviour change, so it would carry a
  §9 entry and its own long run.
- With it, I would ask ADR-0028 §1's amendment to the figure reached only for the Detailed worlds
  (village of 500 1.38×, three of 400 1.31×), whose cost is spread over many features and has no
  one cut.
