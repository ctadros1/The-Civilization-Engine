# Judging a failing source and searching for ore: depletion, giving up and prospecting for The Civilization Engine

## Executive conclusion

**Judge a source that only falls by what it gives now, not by what it gave on average; and treat a search for ore as the covering of ground, whose worth falls as the ground is covered and ends when it is.** This short report was written for one design question (ADR-0043), after the M11 demo showed two faults of memory (plan §7, M11d's ninth and tenth parts): a household believed a patch of bog ore it had emptied still gave 1.28 units an hour, and two households walked 4,030 and 13,320 person-hours in a year looking for copper that lay beyond their reach. It reads the behavioural ecology of patch use, search theory and what is known of prospectors' practice, briefly (plan §1: plausible, not proven). Its findings:

1. **A forager leaves a patch when what the patch gives at the margin falls to what elsewhere gives.** This is Charnov's marginal value theorem (1976). Its qualitative predictions hold widely; animals often stay somewhat longer than it predicts. What matters for a decision is the patch's *current* return, not its mean since first visited.
2. **How much weight old experience deserves depends on how fast the world changes.** Learning-rule and Bayesian models of foraging discount older returns at a rate fitted to the environment (McNamara and Houston 1987). For a stock that never renews and that the forager itself draws down, every earlier sight is of a richer place than the one that exists now: the latest sight carries all that is worth knowing, and it never goes stale until the place is worked again.
3. **What a forager leaves behind is a measure of the patch as it was left** (the giving-up density). A worked-out source is remembered as worked out; prospectors and diggers "moved on" when a strike "played out", and did not go back.
4. **A search over ground finds a showing body with a probability that rises with the ground swept.** Koopman's random-search law gives the chance of detection as `1 − exp(−W·L/A)` for a sweep width `W`, a track length `L` and an area `A`. After an unsuccessful search, the chance that a body lies in the ground searched falls; when bodies are rare, the chance that one lies in the ground *not yet* searched hardly changes. A walk's worth is therefore proportional to the fresh ground it covers, which, for walks drawn over the whole area, is the share not yet looked over.
5. **People searched by looking over ground and remembering it.** Agricola (1556) describes looking for outcrops, washed ore in streams and the effects of ore on plants, and trenching until a vein is found. Nineteenth-century prospecting was traversing, panning and outcrop inspection, creek by creek. News of finds travelled fast and drew searchers away; news of barren ground mostly stayed with those who had searched it.

Throughout, **observed** findings are kept apart from **proposed TCE rules**. Several sources were reached only as search-engine summaries in this session; they are marked so (*search summary*), and none is load-bearing without a design prior's label.

---

## 1. Mechanisms: causal rules TCE can implement

### 1.1 A patch is judged by what it gives now

**Observed.**
- Charnov's marginal value theorem: when returns within a patch fall as it is used, a forager maximising its gain over time should leave when the marginal rate of gain in the patch falls to the average rate of the habitat, travel included (Charnov 1976, *Theoretical Population Biology* 9: 129–136; [overview](https://en.wikipedia.org/wiki/Marginal_value_theorem), *search summary*). The *giving-up time* (time since the last item found) and the *giving-up density* (what is left when the forager leaves) are its field measures ([JoVE: optimal foraging](https://www.jove.com/science-education/10613/optimal-foraging), *search summary*).
- Empirical support is mostly qualitative: foragers leave poorer patches sooner and stay longer when travel is longer, but often exploit a patch further than the theorem predicts (same sources, *search summary*).
- 03-06 §1.2 already asks TCE to make expected returns depend on "remembered patch abundance" and to separate in-patch speed from door-to-door productivity. 08-01 §1 asks that gathering a productive patch today affect tomorrow's opportunities.

**TCE rule (proposed).** A gathering decision weighs each place by the return it is expected to give now, against the walk. The existing decision already compares places by expected return for the walk (plan §7, M1); what changes is how "expected now" is formed for a stock that only falls (§1.2).

### 1.2 How much the past counts depends on how the stock changes

**Observed.**
- Foraging models that learn patch quality from experience use a linear-operator (exponentially weighted) rule, in which a learning rate sets how far the current return outweighs past experience (Bush and Mosteller 1955; Kacelnik and Krebs 1985; McNamara and Houston 1987, *Journal of Theoretical Biology* 125: 385–395, "Memory and the efficient use of information"). McNamara and Houston showed that such an operator can track a parameter that changes over time, and that the right emphasis on new experience depends on how fast the environment changes ([Eliassen et al. 2009](https://bio.uib.no/te/papers/Eliassen_2009_Quantifying_the_adaptive.pdf), *search summary*).
- Marshall et al. (2013), with 54 chacma baboons, found a Bayesian-updating model fitted patch residence best in an experiment and the "prescient" marginal value model best in natural foraging, and read this as foragers weighing recent experience where the environment is predictable; the effect of a single recent experience was weak ([Exeter repository](https://ore.exeter.ac.uk/repository/handle/10871/19089), *search summary*). In starlings, travel experience before the most recent had no effect on load size (Cuthill et al., [reading](https://home.cs.colorado.edu/~mozer/Teaching/syllabi/7782/readings/cuthill%20etal.pdf), *search summary*).
- TCE today remembers a place by the mean return of every hour worked there, the evidence fading over the resource's renewal time and at most ten years (`KnownPatch`; M5a). That is right for a stock that regrows: an old sight is of a place that has since recovered toward what land of its kind gives. It is wrong for a stock that never regrows and that the household itself takes: an old sight is of a place since emptied by exactly what was taken.

**Reasoning (not a measurement).** For a stock that only falls, the stock at the last sight bounds every later state from above, and the household's own takings since are the only change it could know of. The last sight, of what was *left*, is a sufficient summary; averaging in earlier, richer sights only biases the belief upward. A short-memory exponentially weighted mean is the same rule with a learning rate below one; with the stock falling monotonically there is no noise to average away except the gatherer's own skill, which a sight of what is left at a standard pace removes.

**TCE rule (proposed).** For a resource that does not renew, a place is remembered by the return that as many hours more would have given as it was last left, and that memory does not fade back toward what land of its kind gives: it is still worked out a generation later, until someone works it again. For a resource that renews, the rule stays as it is.

### 1.3 A worked-out source is abandoned, not revisited

**Observed.**
- Placer fields were "worked out" and their diggers "moved on"; "as soon as a strike played out, miners moved on" ([Mojave Desert history](https://mojavedesert.baremetal.com/people/prospectors-and-miners.html), *search summary*); by 1864 nearly all the diggers at Goldstream, Vancouver Island, had moved to the nearby Leech River (same search, *search summary*). By the late 1850s California newspapers lamented that placers were increasingly worked out and fewer new deposits found (*search summary*).
- New ground was tried with a pan before it was worked; a digger washed about fifty pans a day ([Britannica video transcript](https://www.britannica.com/video/72936/life-men-mining-camp-height-California-Gold), *search summary*). A pan test is a sight of what a place gives now.

**TCE rule (proposed).** Nothing beyond §1.2: a place remembered by what was left as it was last left is abandoned when that no longer repays the walk, and is not walked back to for nothing.

### 1.4 Searching for a body is covering ground

**Observed.**
- Agricola, *De re metallica* (1556), Book II: look for outcropping veins, search streams for ore washed down from veins, note the effects of metals on the plants above them, and dig trenches until an outcrop of a vein is found; he describes dowsing and rejects it ([summary](https://en.wikipedia.org/wiki/De_re_metallica); [ASEG newsletter](https://www.aseg.org.au/sites/default/files/flipbooks/aseg-newsletter-issue-209/inc/html/54.html), *search summaries*). The ASEG note lists three sixteenth-century techniques: visual observation, costeaning (pitting and trenching) and divining.
- Traditional prospecting "involves a considerable amount of traversing (traditionally on foot or on horseback), panning, sifting and outcrop investigation", combing creek beds and ridgelines ([Prospecting](https://dlab.epfl.ch/wikispeedia/wpcd/wp/p/Prospecting.htm), *search summary*).
- 03-05 §1.3: "Observation reveals indications, not a complete deposit outline. Prospecting reduces uncertainty through sampled outcrops, pits, trenches, and later boreholes." ADR-0010 §1: a showing body is found when a walker's route passes within 50 m of it (`FIND_M`, a tuning value).
- Koopman's random-search law: a searcher sweeping a strip of effective width `W` along a track of length `L` in an area `A`, the target placed at random, detects it with probability `1 − exp(−W·L/A)`; `W·L/A` is the *coverage* ([ORSJ paper](https://orsj.org/wp-content/or-archives50/pdf/e_mag/Vol.45_01_013.pdf); [DTIC report](https://apps.dtic.mil/sti/pdfs/ADA014558.pdf), *search summaries*). Bayesian search theory starts from a prior probability that the target is in each cell and updates it after an unsuccessful search; Koopman's optimal allocation is updated stage by stage from the posterior ([MH370 search paper](https://arxiv.org/pdf/2004.14110), *search summary*). The update itself, `p' = p(1 − q) / (1 − p·q)` for a cell searched with detection probability `q`, is standard but was not confirmed against Stone's *Theory of Optimal Search* in this session.

**Reasoning (not a measurement).** Let `p` be the chance that a showing body lies in reachable ground of the right kind, and let a share `c` of that ground have been looked over without finding one (detection near certain within the swept strip, as ADR-0010 has it). The chance a body lies in the unsearched remainder is `p(1 − c) / (1 − p·c)`; a further walk that sweeps a share `Δ` of the whole, of which only `Δ(1 − c)` is fresh when the walk is drawn over the whole ground, finds one with probability `p·Δ(1 − c) / (1 − p·c)`. For rare bodies (`p` small) that is about `p·Δ(1 − c)`: **a walk's hope falls in proportion to the share already looked over and is nil when all of it has been**. For a common body (`p` near 1) the hope of fresh ground rises as the search narrows; that case does not arise in TCE, where a settlement that knows a body within reach does not prospect.

**TCE rule (proposed).** A settlement remembers the ground its prospectors have looked over. A prospecting walk's hope is the flat prior it has today times the share of reachable ground of the right kind not yet looked over; the walk is drawn toward ground not yet looked over; when all is covered, nobody looks. Ground looked over stays looked over: a showing body does not appear later, except where an earthwork cuts into a buried one, which finds it anyway (ADR-0010 §1).

### 1.5 What is remembered, and by whom

**Observed.** Finds travelled as news and drew prospectors away from older fields (the Klondike's prospectors left for Nome in 1899; *search summary* via the same results). Inexperienced parties hastily marked out ground in New Zealand and as hastily abandoned it ([Waikato research commons](https://researchcommons.waikato.ac.nz/server/api/core/bitstreams/0c022c07-cc97-4820-ae3a-47b5ee2c94cf/content), *search summary*). No source found in this session says how barren ground was remembered or shared; it is reasonable that a party, a camp or a village knew where it had looked.

**TCE rule (proposed).** Ground looked over is a settlement's memory, as known places and known deposits already are (households of a settlement talk at the hearth; ADR-0010 §1). Word of mouth between settlements is left for when it is cheap: a find already becomes known by walking past it.

---

## 2. Parameters

**Confidence key**, as in 12-02 §2: **H** strong support for the measurement or formal definition; **M** useful but context-dependent, or confirmed only from a secondary or search-summary source; **L** thin.

| Parameter | Value | Use and limits | Source; confidence |
| --- | --- | --- | --- |
| Leaving rule | Leave when the marginal rate falls to the habitat's average (travel included) | Qualitative; animals stay longer than predicted | Charnov 1976; **H** as theory, **M** as fit |
| Weight of a single recent experience | Weak in natural foraging (baboons) | Learning rates differ with environment | Marshall et al. 2013; **M** (*search summary*) |
| Random-search detection | `1 − exp(−W·L/A)` | Target at random, sweep width `W` | Koopman 1946 via secondary sources; **H** as formula |
| Pans washed a day | ≈ 50 | One digger, placer ground; a pace of sampling | Britannica transcript; **L** (*search summary*) |
| Bog ore half-rate stock, rate | 10 units/ha; 6 units/h | TCE content (`temperate_valley`), tuning values | TCE; — |
| Detection distance from a route | 50 m | TCE `FIND_M`, a tuning value | ADR-0010; — |

### 2.1 Design priors (not measurements)

| Prior | Starting value | Range to try | Why |
| --- | --- | --- | --- |
| Memory of a non-renewing place | The last sight of what was left replaces earlier ones; never fades | An exponentially weighted mean with a learning rate of 0.5–1 a visit | §1.2: the stock only falls; earlier sights are of a richer place |
| What the sight is of | What as many hours more would give a capable adult there, as land of its kind is expected to | The visit's own last hour | Comparable with what land of its kind is expected to give (the decision's prior) |
| Ground looked over, block side | 100 m (twice the 50 m `FIND_M`) | 50–200 m | A route sees about 50 m either side; a block crossed is about seen |
| Hope of a prospecting walk | `PROSPECT_HOPE` (1.0 load, unchanged) × share of reachable right-kind ground not yet looked over | Exponent 1–2 on the share | §1.4 for rare bodies; the flat 1.0 remains a nudge |
| Whose memory | The settlement's (a household of none: its own) | Per household | As known places and deposits; one prospector's walk spares the village's |
| Fading of ground looked over | None | Fade over a generation (25 years) if lived worlds show ground never looked at again | A showing body does not appear later |

### 2.2 Worked example: Stonefield's year of looking

Plan §7, M11d's tenth part: Stonefield's wanter walked about 36 person-hours a day for copper, 13,320 in a year. Its digging walk is 60 minutes off the trails: about 3 km at TCE's easy pace off trails (1.4 m/s × 0.6). The disc within it is about 28 km², some 2,800 blocks of 100 m. If a third lies on the slope copper bodies are laid on, about 900 blocks of the right kind lie within reach. A loop out, across and home of two to two and a half hours covers 6–7.5 km, crossing 60–75 blocks, a third of them of the right kind. Loops drawn over the whole ground cover it as `1 − exp(−n·25/900)`: four-fifths after about 60 loops, nineteen-twentieths after about 110, which is 150–275 person-hours, against 13,320. Since the hope falls with the share covered, the walks would stop before that, as their worth falls below their walking.

---

## 3. Historical and regional variation

- **Foragers and gatherers of renewing stocks** return to patches as they recover; their memory fades with the stock's renewal, as TCE already has it (03-06 §5.3).
- **Diggers of placers and quarriers of surface stone** work non-renewing stocks: they abandon worked-out ground, and the ground is often visible as such (spoil, pits). A worked-out place is recognisable to anyone, which supports remembering it for good.
- **Prospecting effort** varies with institutions: a lone prospector, a party grubstaked by a merchant, a crown or company survey (03-05 §5.4 puts campaigns with institutions). TCE has only a household's walks; organised campaigns, maps of ground searched and paid prospectors are deferred.

---

## 4. Stylized facts and validation targets

| Fact | What TCE should show |
| --- | --- |
| People leave a patch before it is empty when it no longer repays the walk (giving-up density) | A non-renewing patch left with some stock once its return falls below the walk's worth |
| A worked-out source is not walked back to | No trips to an emptied patch that bring almost nothing |
| Most prospecting finds nothing | Prospecting walks that find nothing are common; their sum is bounded |
| Searches end when the ground is covered | Person-hours of prospecting for an absent body level off within months, not years |
| A body within reach is found | A settlement with a showing body in reach finds it as before |

---

## 5. Modeling recommendation for TCE

1. **Keep the existing memory for renewing stocks.** Mean of hours, fading over renewal, ten years at most.
2. **For a stock that does not renew** (`Growth::Deposit`: stone, flint, copper ore, bog ore), remember each place by what was left as it was last left: on each visit, the return as many hours more would give a capable adult there (`Land::yield_around`, without taking), replacing earlier sights, never fading.
3. **Remember ground looked over while prospecting**, per settlement, in coarse blocks of 100 m with the day first looked over. A prospecting walk's hope is today's flat prior times the share of reachable ground of the right kind not yet looked over; its first point is drawn from ground not yet looked over; when none is left, nobody prospects.
4. **Do not** build a probability map of bodies, a geology-informed prior, campaigns or paid prospectors here; they belong with geology (ADR-0010's "Revisit when").

### 5.1 Guards

- Worlds with no non-renewing stock worked and nobody prospecting are unchanged.
- A body within reach is still found by walking past it; memory never hides a body.
- The hope never exceeds today's flat prior; the flat prior stays a logged nudge.

---

## 6. Sources, datasets, and limits of the evidence

### Core sources

- Charnov, E. L. (1976). "Optimal foraging, the marginal value theorem." *Theoretical Population Biology* 9: 129–136.
- McNamara, J. M., and Houston, A. I. (1987). "Memory and the efficient use of information." *Journal of Theoretical Biology* 125: 385–395.
- Marshall, H. H., et al. (2013). "How do foragers decide when to leave a patch? A test of alternative models under natural and experimental conditions." *Journal of Animal Ecology* ([repository](https://ore.exeter.ac.uk/repository/handle/10871/19089)).
- Koopman, B. O. (1946, 1980). *Search and Screening*; Stone, L. D. (1975). *Theory of Optimal Search* (not consulted directly).
- Agricola, G. (1556). *De re metallica*, Book II (Hoover translation, 1912).
- TCE's own: 03-05 §1.3 and §5; 03-06 §1.2 and §5.3; 08-01 §1; ADR-0010 §1.

### Limits of this review

- **Search summaries.** Every web source was read only as a search-engine summary in this session; the figures and quotations marked *search summary* should be checked against full texts before an ADR relies on them as measurements. None is used here as more than a direction.
- **No figure was found** for how often prospecting found an outcrop, how much ground a prospector covered in a day, or how barren ground was recorded and shared. The block size, the hope and the memory's scope are design priors.
- **The Bayesian update** was taken from general knowledge, not confirmed against a text in this session.

### What should remain uncertain

- **How optimistic people are about ground not yet searched.** The flat hope of one load is a nudge, not a finding.
- **Whether villages shared news of barren ground with neighbours.** Left out until it is cheap and a demo asks for it.

**Bottom line:** remember what was left of a stock that only falls, and remember where you have looked. Both are small memories, and both are what people who dug and searched for a living kept.
