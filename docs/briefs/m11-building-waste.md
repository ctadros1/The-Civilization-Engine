# M11 design brief: modern construction, refuse and residues

**Scope.** Plan §7's M11 lists "modern architecture primitives, waste" beside electricity, motor
vehicles, transit and telecom, which ADR-0036 decided. This brief covers the two left: new
structural systems and their materials (11-14), and refuse, its disposal and the residues of the
works (12-10). Written 2026-10-30, by Claude under the owner's standing authority, from reports
written the same day.

**Read:** 11-14 (modern construction) in full; 12-10 (modern waste) in full; 12-02 §1.4–§1.5,
§5.2–§5.5 again for what 12-10 builds on. Citations are to report sections. Both reports mark
figures taken from search summaries (M or L); none of those is load-bearing below without a design
prior's label.

## What the code already has

- **Buildings:** frame grammar version 3 (ADR-0031 §1): up to four storeys, mass walls by storey
  that carry the walls above, openings, a stair, trussed roofs and storeyed posts opened by
  techniques; checks of bending, buckling, crushing and a lateral mode in `structure.rs`; fire by
  stages in minutes with goods burned under the roof (ADR-0022 §4–§5); rule templates such as
  `MostStoreys` and `midden_from_wells`.
- **Where to build:** a household weighs plots in hours a year (ADR-0030 §1); a building's height
  is a choice in those hours (ADR-0031 §6). Households build their own homes; nobody lets a storey.
- **The works:** recipes in batches and by elapsed days, power at a site, engines that burn a named
  fuel, a dynamo (ADR-0033, ADR-0036); `pig_iron`, `blast_furnace`, `coke`, `quicklime`, a kiln.
- **Waste:** one midden per household, 0.5 kg a person a day, decaying with a 365-day half-life,
  dunged onto fields, carried from the water under `clear_middens`; contamination as directed
  links from a heap to wells and reaches (`contagion.rs`). No sewers.
- **Runs and fares** on track and road (ADR-0034 §4–§5, ADR-0036 §7); ledger channels by purpose.

## 1. Modern construction (11-14)

1. **Frames change which limit governs** (11-14 §1.1, §1.3). Under masonry, walls thicken with
   height and eat floor (NYC 1892: 4 in more every 15 ft down; the Monadnock's 1.8 m base at 16
   storeys, §2.6) long before brick crushes. Under a frame each storey's enclosure is carried by
   the frame, so walls stay thin; height is then limited by stiffness, fire, foundations and lifts.
2. **Each material has its own failure** (§1.2, §2.1): steel yields and buckles; concrete is strong
   in compression only and gains strength with age (about 0.74 of its 28-day value at 7 days);
   reinforced concrete puts bar where tension is.
3. **Materials come from long chains** (§1.7, §2.2): Portland clinker from limestone and clay burnt
   hotter than lime (about 1.7 t of raw material a tonne of clinker), then ground with power; steel
   from pig iron in a converter or hearth; sections and bar from a powered rolling mill.
4. **Unprotected steel loses strength in fire** (§1.6): about 60 % of its yield at 550 °C and 40 % at
   600 °C; protection is a property of the member, in minutes.
5. **People built tall where floor near activity was dear and someone could fill the floors**
   (§1.10, §5.6): Chicago's height-to-land-price elasticity about 0.3–0.45. TCE already prices
   ground in hours; what it lacks is anyone who lets floors.
6. **Lifts are a capacity that takes floor** (§1.5, §2.4); wind governs tall frames (§1.4).

**My proposal.** A frame version 4 beside the frozen three, not a new grammar family: 11-14 §5.2
proposes a `tower` grammar, but v4 reuses v3's assembly, stages, checks and renderer kit, and a
frozen version is how v3 was added beside v1 and v2 (ADR-0031 §1). Two post systems: a metal
skeleton and a reinforced-concrete frame, each opened by a technique, with each storey's enclosure
carried by the frame. Materials as recipes on M10's chain. Steel's and concrete's own checks, and
fire's effect on metal. **Deferred, by the time box:** lifts and wind wait for anyone to build
tall, and building tall waits for letting storeys (a residence change, M8 brief open question 7);
glass, curtain walls, welding, prestressing and piles wait too. With nobody letting floors, the
likeliest first use of a frame is a wide or fire-prone works, as in history's iron-framed mills
(11-14 §3.2), and the likeliest outcome in a lived world is no frame at all: a correct outcome
(11-14 §5.6).

## 2. Refuse and residues (12-10)

1. **Refuse follows fuel** (12-10 §1.1, §2.1): coal leaves about 11 % ash against 0.3–1 % for wood,
   and ash was 57–91 % of British household refuse between about 1919 and 1935 and about 83 % of
   Manhattan's around 1900–1920. Ash is inert: it does not rot.
2. **Mass is conserved** (12-02 §5.2; 12-10 §5.2, §5.11): today's heap decays its whole mass, so a
   coal town's ash would vanish; `make_gas` loses 0.21 kg of every kilogram of coal; the pig-iron
   recipes leave no slag.
3. **Collection is logistics with a destination** (12-10 §1.3, §5.3), about half to two-thirds of
   what a town spends on refuse; a round cannot load without somewhere to put it.
4. **Tips are places with capacity and neighbours** (§1.4, §5.4): open, covered or burning; what
   washes from them follows the same links as a heap's.
5. **A destructor burns refuse as a poor fuel** (§1.5, §5.5): 25–35 % clinker left, 15–80 kWh a
   tonne where it drove a dynamo.
6. **Laws came from seen nuisance** (§1.9, §5.9): occupiers' duty to clear, then a public round.

**My proposal.** The heap gains an inert share fed by the ash of what the household burns; fuels
carry an ash share. Residues are goods on the recipes that make them, so mass closes. A tip is a
site record that takes refuse and residues and joins the contagion links by its organic load.
A household firm may post a refuse round, paid per collection; a polity may name a tip and ask
refuse carried there. A destructor is an engine whose fuel is refuse. **Deferred:** sewers and
treatment (no sewer in the kernel), salvage trades beyond what a recipe already takes, toxicity of
residues, hog feeding (no beasts), compaction tools.

## 3. Guards

- A world where nobody knows the new techniques lives as before, every lived figure equal, apart
  from the heap's ash share, which changes what a heap weighs and so what washes from it: that slice
  records new baselines (as ADR-0030 §1 did) and says so.
- Every recipe, run and tip conserves mass; a test sums a world's residues in and out.
- No cleanliness score, pollution radius, era-keyed rate or storey count anywhere.
