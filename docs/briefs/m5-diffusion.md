# M5 design brief: diffusion between settlements

**Scope.** This brief covers how techniques, building styles, ideologies and news pass between settlements through contact:

- who carries them: migrants, spouses who marry between villages, traders, visitors and people hired away;
- what a person must see or do to take something up;
- why a neighbour's way is copied or refused;
- how distance and frequency of contact shape it;
- what the observer sees: where a technique or a style came from;
- how to measure it, and the M5 demo's "one adopts the other's roof style" (plan §7).

Roads, merchants, migration decisions, splinter founding and diplomacy have designs of their own; this brief takes from them only the contacts they create. The scale is 2–3 settlements and 3–8k people (plan §4.3). Reports read in full: 07-02, 11-02, 04-05, 09-16, 05-06 and 08-12, the last two for what travellers carry.

**What exists today** (checked in the code on 2026-10-09):

- **Knowledge** (ADR-0008). Each person's entries are known, learning or heard of, each with its source: founder, upbringing, taught by whom, found, or the observer (`KnowSource`). A settlement's record notes when a technique became known there and when it was lost. Learning needs a household member, or the owners of a workshop that hired the learner, at the same work.
- **Style** (ADR-0009 §7). Each household has a taste (pitch, eaves, overhang) and the building that moved it most; each building records the one it followed. `review_tastes` (`civ-agents/src/population/taste.rs`) meets only buildings of the household's own settlement.
- **Word and ideology** (ADR-0016). Claims pass at home at midnight and between companions at the hearth; an ideology is taken up by trust in the teller times fit.
- **Within one settlement only.** Partners are found only there (`find_partner`, `population/life.rs`); households trade only there; a household that leaves is gone from the world (README).
- **A leak to fix first.** `Target::Hearth` names no settlement, and `hearth_company` (`population/ties.rs`) gathers everyone whose target is `Hearth`. `keep_company` draws ties from that company and calls `share_word`, `share_opinion`, `share_norms` and `share_ideologies` on each pair. As I read it, with two settlements the people at both hearths would form one company, so ties, claims, opinions, norms and ideologies would pass with no contact. M5 must key the hearth by settlement before measuring any diffusion.

Nothing passes between settlements today.

**Where the reports agree.** I treat these as invariants:

1. People or things carry what crosses, on real journeys. Distance acts only through how often and how cheaply they travel, so what travellers carry takes no second distance penalty (07-02 §1.3; 09-16 §1.2, §5.2).
2. Contact spreads opportunities, not capabilities. Awareness, competence and use are separate states with separate clocks. For style they are preference, new building and standing stock (07-02 §1.1–1.2; 11-02, opening; 04-05 §1.1; 08-12 §1.7; 09-16 §1.1).
3. A settlement's knowledge and style summarize its people and buildings. They are never stored flags or vectors that overwrite individuals (07-02 §1.1; 04-05 §5.1; 11-02 §5.5).
4. Adoption is a decision at a real opportunity (a commission, a planting, a hire), not a roll every tick, and exposure need not lead to it (07-02 §1.5, §4; 11-02 §5.3; 04-05 §1.3, §5.3).
5. Resemblance is not evidence of diffusion; similar constraints make similar buildings. Keep provenance (11-02 §1.1, §5.2, §5.5; 04-05 §6).
6. No universal copying rate, conformity strength or diffusion speed transfers to TCE (07-02 §2.1; 11-02 §6.2; 04-05 §2.1, §6).
7. Ten retellings of one origin are one source (09-16 §1.6; 04-05 §1.3; 07-02 §1.4).

**Where they differ, among themselves or from what M3b built.**

- **Costly choices.** 11-02 §1.1 and 09-16 §1.6 offer complex contagion (several reinforcing sources) as an analogy for costly choices. 11-02 §2.2 proposes a partial move per encounter, which M3b built. 04-05 §5.2 proposes a kernel mixing unbiased, conformist, prestige and payoff copying. None measures which applies to roofs.
- **Conformity.** 04-05 §2.3 proposes a frequency exponent of 1.3, while 04-05 §2.1 finds no transferable estimate. 04-05 §1.3 defines conformity for discrete variants; TCE's style traits are continuous.
- **Exemplars kept.** 11-02 §5.5 suggests 5–20 salient exemplars a person. M3b keeps one admired building per household.
- **When taste is reconsidered.** 04-05 §2.3 proposes 0.5 reviews a year (0.1–2.0); 11-02 §2.2 proposes a move per meaningful encounter. M3b's yearly review sits between them.

## 1. Mechanisms

| Mechanism | Saved state (new unless marked) | Driven by |
|---|---|---|
| Residence apart from presence | A person away keeps household and settlement; where they are is their trip and activity (exists) | Visits, hired work, trading trips (05-06 §1.1) |
| A hearth per settlement | The hearth target names its settlement | Company at the hearth |
| Moves between settlements | The household's settlement changes; people keep what they know, their taste and what they admired, ideologies and ties | Migration and marriage decisions (other designs) |
| Knowledge provenance | A `Known` event names the settlement its knower came from; a `Lost` event says whether a settlement tied to this one still knows the technique; an awareness source `Seen` with the settlement | Arrivals, seeing work or goods, deaths and departures |
| Buildings seen elsewhere | Per person, up to 8 buildings of other settlements, newest first, cleared at the household's review | Coming within sight of a building on a trip |
| Taste provenance | A household's `admired` and a building's `followed` (exist), which may now be elsewhere | The yearly review |
| Claims about elsewhere | The claim's settlement, and the hearing record's teller and origin (exist) | Home at midnight; company at the hearth |
| Ideology provenance | Who a holding came from (exists) | The hearth |
| Contact counts | Per pair of settlements a year: visits, person-days present, moves, marriages, trades | Each such event |

**Derived, not saved:**
- each settlement's spread of taste, of new buildings' traits and of standing stock;
- its founding way of building, recomputable from the seed (`founding_taste` keys it by band);
- knowers per technique, and diffusion curves;
- culture labels (04-05 §5.1; 11-02 §5.5).

**Size:** 8,000 people × 8 buildings seen × 16 B ≈ 1 MB, and transient (my arithmetic).

### 1.1 Carriers

**What the reports say.**
- **Migrants** carry practised routines and teaching capacity. Whether an industry follows depends on work, inputs and successors (07-02 §1.2). The skills that leave are gone from the origin (05-06 §1.5), and migration does not mechanically erase differences (04-05 §1.7).
- **Marriage** is a learning route (04-05 §1.2) and a move with its own logic: kin spread across villages smooth risk (05-06 §1.2). No universal rule says which sex moves (05-06 §3). Daughter settlements stay in their parent's marriage and exchange network (05-06 §1.3, §5.2).
- **Traders** carry objects. An object shows function and style but not hidden steps; buying a steel tool "should not automatically teach steelmaking" (07-02 §1.2; 08-12 §1.7). Traders also carry dated price reports (08-12 §1.6) and news. A merchant is an information bridge because they move between social settings (09-16 §5.2), and long stays build cultural contact (08-12 §1.7).
- **Chain migration.** Earlier migrants give those they know information, lodging and introductions (05-06 §1.2), so corridors grow along relationships, not by settlement-wide updates (05-06 §4).
- **A member back from market** may pass news on; it is not a household-wide update (09-16 §1.3).

**My proposal.**
- **Five carriers:** a household that moves, a spouse who marries in, a person hired at a neighbour's workshop, a trader or buyer on a trip, and a kin visit. Each carries exactly what its person holds; nothing is copied to a settlement.
- **Marriage between settlements.** `find_partner` also considers unpartnered adults elsewhere whom the person has a tie with, met at a hearth or a market. Where the couple lives follows the two households' land and room, not sex. This is not in plan §7's list for M5, but in villages of 50–200 with close kin excluded it may become the commonest carrier (open question 3).
- **Leaving goes somewhere.** A household that gives up may move to a settlement it knows of, taking everything with it. The migration design owns the decision; diffusion needs only that people arrive with their state.
- **A visit is a trip.** A person away stays a member of their household and settlement, and a visit never triggers the loss check (05-06 §1.1).

### 1.2 Techniques

**What the reports say.**
- **Channels yield differently.** Seeing an object, watching its use, watching it made and supervised instruction each teach different amounts (07-02 §1.2).
- **Learning needs practice:** 20–200 learner-hours for a narrow, observable procedure and 500–5,000 for a specialised stage, with 1–3 learners per teacher (07-02 §2.3).
- **Adoption is a choice.** A household compares a technique with the one it uses now, at a real decision point (07-02 §1.5). Many who are exposed never adopt: in one Mozambique study, 48 % of 96 non-adopters did not know the techniques (07-02 §2.1, §4).
- **Loss is local.** Keep "no local practitioner" apart from "no accessible practitioner"; a settlement should look to the network it can reach, not only its residents (07-02 §5.4).
- **Migrant workshops can last** if they train successors (07-02 §4). Expected successors are R = b·T·s (07-02 §5.3).

**My proposal.**
- **Knowing still needs working beside a knower** (ADR-0008 §4). Between settlements that happens three ways:
  - a knower moves in;
  - someone marries into a knower's household;
  - someone is hired at a neighbour's workshop whose owners know the technique (the supervised path that exists).

  Children then learn it at home.
- **Seeing gives awareness only.** A visitor who sees work done that needs a technique they lack, or a household that acquires a good made with one, gets an awareness entry with source `Seen` and the settlement. Awareness already raises the rate of trying toward a technique (ADR-0008 §3's reconstruction route), and that is all it does. Which work counts as seen is a tuning value: work at places within sight of the visitor's way.
- **Adoption stays economic.** A household works by the cheapest recipe its members know (ADR-0008, consequences), so a technique arriving with a spouse is used or not by the same comparison. No new score is needed.
- **Provenance.** A `Known` event caused by an arrival names the settlement the knower came from. It is saved because it cannot be derived once they die.
- **Local and accessible loss.** When a technique is lost, the record notes whether it is still known in a settlement that any remaining resident has a tie to (07-02 §5.4). The observer is shown it; nothing reads it in M5.

### 1.3 Building style

**What the reports say.**
- **Exposure** comes through observed buildings, travel, migration, trade and apprenticeship (11-02 §1.1).
- **Prestige** is specific to the social group and the domain, and is not wealth (11-02 §1.1; 04-05 §1.4).
- **Three clocks:** exposure, implementation and built-stock change (11-02, opening). Record preferences, new projects and standing stock separately (11-02 §4).
- **The commission is the decision.** A collapse or an unpopular patron weakens a design's appeal (11-02 §5.3).
- **Familiarity saturates.** Walking past the same building again is not a new encounter (11-02 §2.2).
- **Copying a look is not copying a structure.** A copied silhouette does not confer the ability to build it (11-02 §1.1, §5.4).
- **The stock lags.** dx/dt = (r+g)(q−x). At 1 % replacement a year and no growth, even universal adoption in new buildings changes only 39 % of the stock in fifty years (11-02 §2.3).

**What M3b built.** Once a year, each household's taste moves `alpha` × admired ÷ `prestige_most` of the way toward each building its settlement finished in the past year. A building's admiration is 1 + 2 × rank, where the rank is the mean of its owner's rank in goods and its craft rank (`taste.rs`, `style.rs`; `alpha` 0.1, `prestige_most` 3).

**My proposal.**
- **Buildings seen elsewhere join the review.** A person who comes within sight of another settlement's building, finished in the past year, notes it (up to 8, a tuning value inside 11-02 §5.5's 5–20). At the yearly review the household meets these alongside its own settlement's, oldest first, each once, by the same rule.
- **A visitor admires what they can know.**
  - The craft half stays as now, ranked among that settlement's year.
  - The patron half comes from the visitor's own esteem of the owner (ADR-0014 §3: esteem is a tie's evidence above the prior, so a stranger scores 0).
  - Whether esteem should replace goods for home buildings too is open question 2: M3b left admiration by reputation for later.
- **Traits are copied by sight; techniques gate programs.** Pitch, eaves and overhang are within every household's know-how, so seeing is enough. A program that needs a technique the household lacks stays out of reach; a frame building needs jointed framing.
- **Provenance crosses.** A household's `admired` and a building's `followed` may name a building elsewhere and survive a move. The readout names the settlement: "after Cal's hut at Westford".

**What the rule implies** (my arithmetic):
- **Taste moves fast.** Each building met moves taste 0.033–0.1 of the gap. Meeting four a year closes about 24 % of the gap to those exemplars a year, a half-life of about 2.5 years. New building and the standing stock are the slow clocks.
- **The neighbour's share.** Under repeated exposure, taste settles near the admiration-weighted mean of what the household meets. With n_home and n_away buildings met a year, the share of the way toward the neighbour's style is about n_away·w_away / (n_home·w_home + n_away·w_away). A stranger's building weighs about 1.5 on average against a home building's 2, giving 0.43 at equal counts and 0.2 at one abroad for every three at home.
- **Symmetric contact meets in the middle.** Two settlements in equal contact move toward a middle way. One taking the other's way needs asymmetry:
  - one settlement larger, with more buildings to see and more people to send;
  - or richer, or better built, and so more admired;
  - or taking in its neighbour's spouses and movers, whose buildings then count as home buildings.

  The demo has to show one of these at work (§3.4).

### 1.4 Ideologies and news

**What the reports say.**
- **Stages are separate.** Hearing, believing, passing on and acting are separate steps. One contact is enough to hear; acting may need several (09-16 §1.1, §1.6).
- **Arrival follows journeys.** News arrives by journeys and waits for a carrier; it does not spread by radius. With a departure every P days, the expected wait is P/2 (09-16 §1.2–1.3).
- **Contact is timed.** Markets and gatherings are timed contact events, so news arrives in bursts and coverage has a long tail (09-16 §1.3, §4).
- **Sharing depends on relevance,** urgency and the relationship with the source (09-16 §1.5). Priors per opportunity: 0.05–0.25 for routine claims, 0.4–0.9 for urgent ones (09-16 §2.2).
- **Distortion waits** until the undistorted system works (09-16 §5.5).
- **Contact is neither always hostile nor always assimilative,** and what crosses differs by domain (04-05 §1.7).

**My proposal.**
- **No new channel.** Once the hearth names its settlement, a visitor at a neighbour's hearth is company like anyone else, and a person back home tells their household at midnight. Both work as now, by the existing keyed chances. ADR-0016 already lists "several polities carry news between settlements" among its revisit triggers.
- **Strangers persuade little.** A visitor starts with no tie (ADR-0014), so trust is low and an ideology passes mainly along ties that cross settlements: kin married out, or trade partners. This is the identity filter; it comes from trust, not from a separate term.
- **Relevance.** A claim about another settlement is told at the routine chance unless the hearer's household has kin or trade there. The chance is a tuning value inside 09-16 §2.2's 0.05–0.25. New kinds of claim (a treaty, a famine, a price) belong to the designs that make them.
- **Laws stay local.** A polity's law binds only its members, and nobody needs to know a neighbour's laws. Copying a neighbour's law is deferred (§4).

### 1.5 Why a neighbour's way is copied or refused

**What the reports say.**
- **Prestige** is a domain-specific cue for whom to learn from: more than 2× the odds in one experiment, almost 5× when the domain matched (04-05 §1.4, §2.1). It is freely conferred, not coerced (11-02 §1.1).
- **Seen success.** Outcomes are observed with noise and discounted when conditions differ (04-05 §1.4; 07-02 §1.4). Borrowed evidence weighs 0.1–1.0 of one's own trial (07-02 §2.3). Visible failure erodes appeal (11-02 §5.3; 04-05 §1.4).
- **Conformity** is copying the common variant among the people actually observed, applied at learning events only (04-05 §1.3). In models it can keep groups apart, but only under specified conditions (04-05 §1.3).
- **Identity and fit.** Ritual or identity fit is one term in a commission's utility (11-02 §5.3). Religious and ideological selection acts on particular requirements, not whole styles (11-02 §1.1). Neither report gives a number.
- **Interests and feasibility.** Resistance comes from particular interests, institutions, costs and local unsuitability, not from backwardness (07-02 §1.5–1.6). Imitation needs the materials, skills and tools (08-12 §1.7; 11-02 §1.1).
- **Saturation.** In one study each extra adopting contact added 5.4 points at first, and the effect turned negative past about 10 contacts (07-02 §2.1).

**My proposal for M5.**
- **Prestige:** admiration as in §1.3, from craft and the viewer's own esteem of the owner.
- **Seen success:** a building that has given way is admired by nobody (exists). A technique's success is its household's own cost comparison once known (exists). No belief about payoffs is kept yet.
- **Conformity:** none added. Home exemplars outnumber foreign ones, so the local way already weighs most. An exponent on continuous traits would be my invention (open question 4).
- **Identity:** no identity term. Refusal comes from few encounters, a stranger's low esteem, an ideology's poor fit with what someone holds dear (exists), and missing techniques or materials. If convergence comes too easily, look at these mechanisms before adding a "foreignness" penalty.

### 1.6 Distance and frequency

**What the reports say.**
- **Distance acts through networks,** and later adopters become sources themselves (07-02 §1.3).
- **A distance kernel only off the network.** exp(−t/τ) is for contacts outside the simulated network, with occasional long links (07-02 §1.3).
- **Network position beats proximity** (09-16 §4; 11-02 §4).
- **Travel.** Ordinary foot travel is about 25 km a day (09-16 §2.1). A porter carries 15–30 kg for 15–30 km a day (08-12 §2.4).
- **No transferable decay.** The estimate of a 73 % fall in contact per 1,000 km is macro-scale and does not transfer to small settlements (07-02 §2.1).

**My proposal.**
- **No distance term in diffusion.** Distance enters only as walking time in the scored choices to visit, trade, hire out, marry and move. A choice's cost already counts the walk, as buying does today (plan §9, "Buying from a neighbour").
- **Day trips only.** A visit is a round trip within a day's walking range (a tuning value). Overnight stays need hosting and wait (§4).
- **Frequency is counted** in the contact counts above, so every diffusion can be read against the contact that carried it.

### 1.7 What the observer sees

- **Knowledge panel.** Each technique's history says where it came from: "known here since spring of year 12, brought by Ada, who married in from Westford" or "seen at Westford by Bo (heard of)". A loss notes "still known at Westford".
- **Building readout and inspector.** A building reads "after Cal's hut at Westford". A person's entry lists the buildings they have seen elsewhere and the chain of followed buildings back to the first link that crossed (11-02 §5.5: commissioned by, influenced by).
- **Style per settlement.** The view shows the founding way, households' taste now, this year's new buildings and the standing stock, each as the mean and spread of pitch, eaves and overhang: the three clocks (11-02 §4). It also gives the share of this year's new buildings that follow one elsewhere.
- **Contacts.** For each pair of settlements: visits, person-days, moves, marriages and trades a year.
- **A claim's coverage.** For each settlement: T10, T50 and T90, or "not reached" (09-16 §4).
- **The kernel writes the words.** The observer only shows them (plan §1, rule 5).

### 1.8 Expensive to reverse

- **Saves.** A hearth target that names its settlement, a new `KnowSource` code and new fields on knowledge events are save changes, appended, never renumbered (ADR-0002).
- **Residence apart from presence** (05-06 §1.1) is decided once for visits, hired work and trading trips.
- **No new ADR.** Fold these into the migration ADR. ADR-0008's revisit trigger ("several settlements trade or migrate") and ADR-0016's are the hooks. The rest is behaviour.

## 2. Numbers worth keeping

| Quantity | Value | Source and standing | Use in M5 |
|---|---|---|---|
| Taste move per meaningful encounter | 0.02–0.20 (built 0.1) | 11-02 §2.2, test prior | Keep |
| Weight of the most admired exemplar | 1–3× (built 3) | 11-02 §2.2, test prior | Keep |
| A new trait per commission | 0.1–3 % (built 1 %) | 11-02 §2.2, test prior | Keep |
| Salient exemplars a person | 5–20 | 11-02 §5.5, a storage choice | 8 buildings seen elsewhere (tuning value) |
| Stock replacement | 0.5–2 % a year; t50 = ln 2/(r+g) | 11-02 §2.2–2.3 | Read off runs, never set |
| A regional style lag | 35–40 years (Tudor Revival in one town) | 11-02 §2.1, moderate | Context only |
| Taste reconsideration | 0.5 a year (0.1–2.0) | 04-05 §2.3, prior | The yearly review stays |
| Demonstrators sampled; conformity exponent | 5 (3–9); 1.3 (0.7–2.0) | 04-05 §2.3, priors | Unused (open question 4) |
| Prestige cue | >2× odds; ~5× domain-matched | 04-05 §2.1, one experiment | Domain-specific esteem |
| Same-village effect on choosing an adviser | OR 1.8–2.76 | 04-05 §2.1, Fiji | Home exemplars weigh more |
| Uptake of visibly useful components | ~70 % within 1–2 trials | 04-05 §2.1, laboratory | Visible traits spread faster than opaque crafts |
| Learner-hours | 20–200 narrow; 500–5,000 specialised | 07-02 §2.3, prior | `learn_h` (exists) |
| Learners per teacher | 1–3 | 07-02 §2.3 | 2 (exists) |
| Weight of borrowed payoff evidence | 0.1–1.0 of one's own trial | 07-02 §2.3, prior | Deferred |
| Adoption reconsideration | 1–2 a year on farms; 4–12 in workshops | 07-02 §2.3, prior | Real decision points only |
| Successors per practitioner | Test 0.5, 1, 2 | 07-02 §2.3, §5.3 | Diagnostic for a migrant's craft |
| Effect of one more adopting contact | +5.4 points; negative past ~10 | 07-02 §2.1, one study | Saturation check |
| Non-adopters lacking know-how | 48 % of 96 | 07-02 §2.1, one study | Exposure is not adoption |
| Logistic 10–90 % interval | 4.394/r; r = 0.05–0.5 gives 88–8.8 years | 07-02 §2.2, arithmetic | Fit to outputs only |
| Foot travel | ~25 km a day; prior 20–40 | 09-16 §2.1–2.2 | Check the visit range |
| Conversations | 2–8 a day | 09-16 §2.2, prior | Hearth sampling (exists) |
| Sharing a claim | 0.05–0.25 routine; 0.4–0.9 urgent | 09-16 §2.2, prior | Claims about elsewhere |
| Retention | 3–14 days passing news; 30–180 days important | 09-16 §2.2, prior | Existing let-go rules |
| Wait for a carrier | P/2 | 09-16 §1.3, arithmetic | Read off trade schedules |
| Completed moves | ~2 per 100 a year (sweep 0.5–10) | 05-06 §5.4, output benchmark | Context for contact counts |
| Village turnover | 38 % left in 12 years (Clayworth) | 05-06 §2.1, moderate | Context only |
| Contact between forager camps | 28 % (forest) and 56 % (coast) of adult pairs in a month | 09-16 §2.1, low transfer | Context only |

## 3. Validation

### 3.1 Tests

1. **Isolation.** Two settlements with no carriers for ten years produce no `Seen` entries, no `followed` link that crosses, no claim heard about the other and no ideology from a teller elsewhere. This catches the hearth leak.
2. **Distance only through contact.** In a test fixture, the same visits at two map distances give the same exposures (07-02 §1.3).
3. **A knower moves.** The new settlement's record notes the technique as known, with its origin. The old one notes its loss only if the mover was the last knower. The mover's children learn the technique at home.
4. **Seeing is not skill.** A visitor who watches frame building becomes aware, never competent, and cannot build a frame.
5. **A visit is not a departure.** No loss check runs, and the visitor's household is unchanged.
6. **One encounter per building.** A building seen on five visits moves a household's taste once (11-02 §2.2).
7. **Asymmetry.** A settlement of 30 that takes in spouses and visitors from one of 150 ends nearer the larger one's way than the reverse. The test asserts the direction, not a threshold.
8. **Retellings.** News of one origin carried across by two visitors counts as one source (09-16 §1.6).
9. **Gate B** (ADR-0011 §5): contact and crossing counts agree between Accelerated and Detailed, with wide tolerance because the events are rare.

### 3.2 Stylized facts (plausibility checks, not targets; plan §1, rule 4)

- **Exposure comes before adoption,** and some who are exposed never adopt (07-02 §4; 08-12 §1.7).
- **Arrival is not scale.** A technique can arrive with one spouse and stay in few hands (07-02 §4).
- **New buildings change before the stock,** and faster in a growing settlement (11-02 §2.3, §4).
- **Coherent at home, different between.** Each settlement keeps a coherent local way, and differences between settlements persist (11-02 §4; 04-05 §4).
- **Network position beats proximity.** With three settlements, the better-connected one learns first (09-16 §4; 07-02 §4).
- **News arrives in bursts** with a long tail of the unreached (09-16 §4).

### 3.3 Measures

- **Diffusion curves.** Track the share of households with someone who knows a technique, and the share of a year's new buildings nearer the neighbour's founding way than the settlement's own. Fit a logistic afterwards and report r and T10→90 = 4.394/r (07-02 §2.2), or "not reached" (09-16 §4). Keep "ever adopted" apart from current use (07-02 §2.2).
- **Rate of change.** Use r_TV for discrete outcomes and r_log for each trait's mean, per settlement and year, always at the same interval (04-05 §2.2).
- **Distance decay.** Across the dashboard's five worlds, settlements lie at different walking times. Plot contacts a year and arrival lag against walking time, and expect lag to follow contact, not distance itself. No coefficient transfers to match against (07-02 §2.1); my proposal.
- **Null.** Without contact, the settlements' trait means wander only by drift and innovation (my proposal). 04-05 §4's Wright–Fisher null applies to discrete variants.

### 3.4 The demo: one settlement adopts the other's roof style

- **Definition** (my proposal). The demo succeeds when, in some year, all three hold:
  - the mean pitch of one settlement's new buildings lies nearer the other's founding way than its own;
  - at least one of those buildings follows, through its chain of `followed` links, a building in the other settlement;
  - the contact that carried it is on record: a marriage, a move or visits.
- **Starting difference** (my arithmetic, ignoring the 45–55° limit). Bands draw their ways 2.5° apart (SD) around 48°, and households 1° around their band's. The difference between two bands' ways therefore has an SD of about 3.5°, and about 40 % of worlds start more than 3° apart; below that, any change hides inside households' own spread. The demo picks its world by this founding draw, visible in year 0, and says so. That is a starting condition, not an outcome.
- **Needs asymmetry.** Expect adoption only with an asymmetry (§1.3); two settlements of unequal size, founded at setup, is the likely form. The trade in the same demo supplies much of the contact.
- **Time box.** If it has not emerged after a week of tuning, nudge a prior and log a `NUDGE:`: visit frequency, `alpha` within 0.02–0.20, or the weight on marrying across settlements. Report what happened, as the M4 demo did (plan §1, rule 3). Never script the move, the marriage or the copy.
- **Show new buildings, not the skyline.** The stock lags new building (11-02 §2.3).

**Risks particular to this codebase.**
- **The hearth leak** above.
- **Route searches** already take 86 % of a slow day's time (plan §9); visits add long routes, so they should be rare and reuse cached routes.
- **Accelerated mode.** A leisure block runs several sessions (ADR-0011 §4), and its company at the hearth is found once, when it starts (`start_work`). A visitor who arrives mid-block is missed, so count exposure per session, not per block.

## 4. What to defer, and why

- **Records and writing as carriers** (07-02 §1.8): there is no writing yet.
- **Beliefs about payoffs,** and waiting for others' trials (07-02 §1.4; §5.2's trial hazard): today a technique is adopted by its own cost. Add them once adoption is a costly, risky investment.
- **Forbidding a technique or a style by law** (plan §5.6: "a society can know something and forbid it"; 07-02 §1.6): the law pipeline could carry it, but nothing yet makes anyone want it. Guilds and lobbying are M7.
- **Copying a neighbour's law:** a sponsor's forecast comes from their own household's lot (ADR-0016 §4), so this needs a payoff belief.
- **Distortion and propaganda** (09-16 §5.5).
- **Whole repertoires, ornament and hybrids** (11-02 §5.1, §5.4): the grammar expands three traits, and ornament waits for frame v1's four reserved parameters.
- **Captured artisans and conquest** (07-02 §1.2): war is M6.
- **Contact beyond the map** and occasional long links (07-02 §1.3): there is no outside world yet.
- **Overnight stays and hosting;** seasonal labour beyond a day's walk.
- **Caution crossing settlements** through news of a collapse: ADR-0009 §6 keeps caution per settlement.
- **Culture as an entity:** labels stay derived (04-05 §5.1; 11-02 §5.5).

## 5. Open questions for the designer

1. **What counts as adopting** a continuous trait: the definition in §3.4, or a fixed share of the gap?
2. **Esteem or goods.** Should the patron half of admiration come from esteem (ADR-0014) for every building, which changes M3b's behaviour at home, or only for buildings elsewhere?
3. **Marriage between settlements** is not in plan §7's list for M5. Is it in, and who moves?
4. **Conformity.** None, or a pull toward the local median on continuous traits?
5. **A first visit.** Does it meet a neighbour's older buildings, or, as at home, only those finished in the past year?
6. **Visits.** Are they a behaviour of their own (kin visits), or only a by-product of trade, hire and marriage?
7. **Cultures.** Plan §4.1 gives communities a style taste vector. ADR-0009 keeps taste per household and caution per settlement "until cultures exist (M5)". Do cultures become stored entities in M5, or stay derived?
8. **Collapse news.** Should news of a collapse elsewhere move a settlement's caution?
9. **Dashboard.** Should it gain a "neighbours" row (nothing crosses without contact, at least one crossing within fifty years with it), or is the demo enough?
