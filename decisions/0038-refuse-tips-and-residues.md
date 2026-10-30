# ADR-0038: Refuse, tips and residues

Status: Accepted (2026-10-30, by Claude under the owner's standing authority)
Date: 2026-10-30
Milestone: M11
Amends: ADR-0021 §4 (a tip joins the contamination links), ADR-0033 (recipes keep their residues;
an engine may burn refuse), ADR-0035 §7 (a law may name where refuse goes)

## Context

Plan §7's M11 lists "waste". Its research is 12-10 (modern waste), which builds on 12-02, digested
with 11-14 in `docs/briefs/m11-building-waste.md`. Refuse follows fuel: coal leaves about 11 % ash
against well under 1 % for wood, and ash was most of a coal town's refuse (12-10 §1.1, §2.1). Ash
is inert. Today's household heap decays its whole mass with one half-life, `make_gas` loses 0.21 kg
of each kilogram of coal, and the pig-iron recipes leave no slag, so mass does not close (12-02
§5.2; 12-10 §5.2). Collection is logistics with a destination (12-10 §1.3); tips have capacity and
neighbours (§1.4); destructors burned refuse as a poor fuel (§1.5); laws followed seen nuisance
(§1.9).

## Decision

### 1. Parts

M11f builds this ADR in four parts: the heap's ash and the works' residues (§2), tips and the law
that names one (§3), the refuse round (§4), and the destructor with the observer (§5).

### 2. The heap's ash, and residues as goods

- **The heap gains an inert share.** `Midden` keeps its decaying share (with its nitrogen and
  half-life) and adds kilograms of ash that never decay, are not dung and carry no load to water.
- **Fuels carry an ash share** (content, `ash_share` on a good: firewood 0.005, coal 0.11, coke and
  charcoal their own; 12-10 §2.1–§2.2). What a household burns adds its ash to its heap; what a
  works burns adds it to the works' store as `ash`.
- **Recipes keep their residues as goods:** `make_gas` gives tar and ammoniacal liquor besides gas
  and coke (12-10 §5.7); smelting and steelmaking give `slag`. Residues are held like any good,
  take store floor, and nobody wants them unless a recipe takes them.
- This changes what heaps weigh in every world, and so the store floor a works needs: the slice
  records new baselines and says so. What washes from a heap is unchanged, since ash carries none.

### 3. Tips, and a law that names one

- **A tip is a site record:** a place, who opened it (a household, a firm or a polity), kilograms
  tipped by kind (decaying refuse, ash, each residue), and the day it was opened. It is a node of
  the contamination links (ADR-0021 §4) by its decaying share alone, as a heap is.
- **Who tips.** A holder over its store floor, or under a law, may carry refuse or residues to a
  tip it may use: its own, on ground it holds, or one a polity opened. Carrying is walked and
  timed as `clear_middens`'s carrying is.
- **`refuse_to_tip`** (a `PolicyKind`, appended) names a tip and asks households to carry their
  heap's ash and decaying share there each `every_days`; it answers `source_suspected` as
  `clear_middens` does and a new issue, `heaps_near` (a household's heap grew past a content size
  near others' homes). Kept by choice; breaches come only as cases (ADR-0025).
- **No tip without a place:** opening one needs clear ground a holder may use; its siting obeys
  rules the opener keeps (`midden_from_wells`).

### 4. The refuse round

- **A household firm with a vehicle may post a refuse round**, as a road line posts runs (ADR-0036
  §7): the homes it calls at, its days, and the tip it carries to. A home on a round pays a fee per
  call in food on a new ledger channel, `Refuse`, and its heap is emptied into the cart.
- **Households weigh it in hours:** carrying their own heap against the fee by their `hour_kcal`;
  under `refuse_to_tip` the carrying is what the law asks.
- **A polity may run one by law** (`collect_refuse`, appended), paid from its store as common work
  (ADR-0035's `pay_work`).
- **No round loads without its tip.**

### 5. The destructor

- **A destructor is an engine whose fuel is refuse** (ADR-0033's engines): it burns a heap's
  decaying share at a content energy (a tenth to a fifth of coal's, 12-10 §2.2), drives a shaft and
  so may drive a dynamo (ADR-0036 §2), and leaves clinker (25–35 % of what it burns, a design
  prior).
- Its keeper weighs it as a station is weighed: by what its shaft or connections would repay.

### 6. Deferred, by the time box

Sewers and treatment (no sewer in the kernel; 12-10 §5.8 gives their shape); salvage trades beyond
what a recipe already takes; toxicity of residues; covering, compaction and burning tips; hog
feeding (no beasts).

## Consequences

- Mass closes: every recipe, round and tip conserves it, and a test sums it.
- Coal-burning works pile ash and slag that must go somewhere; where nobody burns coal, little
  changes beyond each heap's ash.
- A town may never post a round or pass a law: households carrying their own heap is a correct
  outcome.

## Alternatives considered

- **A cleanliness score or a nuisance radius.** Forbidden (12-02 §5.1, 12-10 §5.10).
- **Residues vanishing at the works.** Simpler, but it breaks the ledger (12-02 §5.2).
- **Collection as a service a polity simply provides.** Collection is runs, hours and fees.

## Revisit when

- Sewers are designed: treatment follows 12-10 §5.8.
- Beasts are kept: garbage as feed, and the laws against it.
- A non-pathogen harm model exists: residues get a dose.
