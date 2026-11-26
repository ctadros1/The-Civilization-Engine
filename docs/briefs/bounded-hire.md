# Brief: a seeker of work weighs the offers within reach of what it knows

Status: **candidate built on `claude/bounded-hire`, not promoted** (2026-11-26). Plan §8 24,
seventh item (the 100k bench workload). Proposed and revised for the owner's reviews of 2026-11-25; implementation approved by the owner on 2026-11-26
with decisions that override this brief where they differ (§9): farm hiring only; an attention
budget of eight distinct employers, at most four of them contacts; at most sixteen offers worked
out including retries; deduplication and reachability before a slot is spent; reachable
neighbouring employers kept; a fresh household-and-day-keyed draw in place of §4.4's rotating
window, with no promised discovery deadline; no vacancy reports; the kept list's invalidation as
§4.2a. §9 records what was built and how it was validated. Nothing here authorizes promotion, a
parameter change beyond the two provisional settings, a limit amendment or a save-format change.

## 1. What happens now (read from the code)

- **Who is weighed.** `best_job` (`population/firm.rs`) works out the offer of every household of
  the seeker's settlement (`households_in`), then of every other settlement's household with a
  field whose centre lies within a straight line of home (`fields_near`, bounded by
  `hire_scan_m`: the work's 30 minutes, `core:activity/hired_work`'s `max_walk_minutes`, at the
  fastest pace walking allows). Employers are already taken once each (the neighbours' list is
  sorted and deduplicated); the cost is that every one has its offer worked out.
- **What an offer is.** `reap_offer_with` (`population/reapers.rs`) gives one offer per employer,
  the first that applies: reaping (`reaping_offer`), else threshing (`thresh_offer`), else
  preparing and sowing (`sowing_offer`), each from all its fields, its hands at a peak's hours and
  the hours hired hands already set out (`hired_set_out`). Nothing is kept for it: field employers
  post nothing, unlike a workshop, which posts a wage (`Firm::wage`, `WageOffer`).
- **Navigation.** Reachability is real, not straight-line: the walk is read from the travel field
  of the settlement's hearth (`walking_field_of`, `home_field`: a Dijkstra fill of the walking
  grid, `NavGrid::travel_field`, as far as the longest walk any activity allows and a fifth more,
  `field_reach_seconds`), rebuilt when the walking grid's travel revision changes, which a flood or
  a crossing changes; a home cut off by a closed ford gets its own (`walking_field_of`). The check
  is one lookup (`TravelField::seconds_to`), made **after** the offer is worked out, on the offer's
  place only: the field ripe longest (reaping), the lowest-id field (sowing), or the employer's home
  (threshing). Another settlement's employer is kept only within the work's walk; the seeker's own
  settlement's within the travel field. `walk_hours` (`population/moving.rs`), the straight line
  over top speed, is moving's and is not used in hiring.
- **How often.** Once per household view (`hh_view.job`): every decision in Detailed mode, from
  the first need after midnight until midnight or a consequential step in Accelerated mode
  (ADR-0011 §4; paid work done refreshes every household's view), once a day in a
  household-resolved day (`resolution.rs`, `ONCE`).
- **Payment capacity.** Reaping's hours are capped by all the crop's grain held, sowing's by the
  grain spare beyond what feeds the household to its next harvest (`spare_of`); threshing's are not
  capped at the offer, and are paid out of the grain threshed, no more than held (`hired_threshing`).
- **Wages.** Only reaping's rises: `reap_wage` bids the share up by `HARVEST_BID_PER_DAY` (5 %) for
  each day the crop has stood ripe, a clock on the standing crop, not a count of hours untaken.
  Threshing's is a fixed share over `thresh_h_per_kg`; sowing's (`sow_wage`) is the share of the
  field's expected crop over its remaining hours, with no bid-up. So the escalation is evidence for
  reaping alone.
- **What is recorded.** The dashboard's Harvest margins row (`ledger.rs` `Margins`) counts hired
  reaping only; hired threshing and sowing are seen only in the ledger's wage channel.
- **Workshops.** Workshop hiring shares `best_job` but not its list: the open, not-owing firms of
  the seeker's own settlement, each checked for tools and reach and ranked with the field offers.

## 2. What the research asks

- 08-10 §5.3, "Use bounded local matching": "Workers inspect a small set of reachable offers
  through local and social networks", and "Unfilled vacancies and hiring losses inform later wage
  revisions", with EURACE-family models as precedent ("workers search subsets of offers").
- 08-10 §5.6 gives "3–8 candidate offers per search event" and a weekly routine review with search
  triggered by events, as "proposed starting settings, not historical estimates".
- 08-04, "Search and matching: bounded, local, persistent": "Use remembered suppliers and
  employers, supplemented by a small amount of exploration"; its table's 3–8 new job candidates
  per search are "low-confidence starting ranges", and "agriculture, casual labor, and crises need
  their own policies". 08-04, "Wages and hiring": raise an offered wage when a vacancy stays
  unfilled and the expected production can support it, not when the obstacle is impassable
  transport.
- 08-10 §1.3: a wage offer is weighed against the value and urgency of the household's own work.
- 05-06 §5.5: settlement-level caches for known wages and vacancies, with individual beliefs and
  access kept.

No report says how harvest hands of the periods modelled learned of work (hiring fairs, word at
the field gate); the first brief's sentence on it had no source and is withdrawn. The research
supports a bounded search; it gives no figure for how many farms one knows of.

## 3. Scope

Farm hiring only: the reaping, threshing and preparing-and-sowing offers `best_job` gathers.
Untouched: the workshop list and its ranking, how an offer is worked out (`reap_offer_with` and
the three under it), wages, payment (`hired_reaping`, `hired_threshing`, `hired_sowing`), who may
work for hire, the observer's `field_hire` (wire 1.115) and words. Workshop hiring would need its
own proposal.

## 4. Proposed design

**4.1 Where candidates come from, and how they change.** Each source is read from saved state at
the decision; none is a fixed list.

1. *Contacts:* the households of the seeker's people's close kin (`kin_households`), and those
   holding anyone its people hold a tie with (`Ties::of`), **in any settlement**. Unlike
   `mend_candidates` (`population/mending.rs`), which keeps kin and ties of the same settlement
   only, and a tie only to the other household's elder because an ask for mending goes to the
   elder, hiring reads a tie to any member: a hand may know a farm's son, not its head. Ties form at
   the hearth (any settlement's, on a visit: ADR-0018 §2), by trade and gifts, and by hire itself
   (`hired_reaping`, `hired_threshing` and `hired_sowing` note `WagesPaid` and `WorkSeen` between the
   worker and the employer's elder), so a past employer is remembered; they fade and are let go
   (48 a person, `[ties]`), so contacts change.
2. *Seen ground:* employers whose fields (for threshing, whose home) lie within the walk, as
   local observation of the fields one passes or can see. This is an approximation: the code keeps
   no record of fields actually passed, so the set is read from the fields' places and the hearth's
   travel field. It changes as fields are marked out, change hands (`fields_moved`) or are cut off.
3. *Word:* none in this slice. The code has no vacancy report; price reports (`reports.rs`,
   ADR-0019 §1) are the pattern one would follow (remaining choice 3).

Of encounters, the code has hearth company (`hearth_company`, three companions a session) and
settlements seen on walks (`places.rs`); both act here only through the ties they make.

**4.2 Reachability before the budget.** Straight-line distance stays a search bound only. An
employer enters the search only if one of its fields' centres (for threshing, its home) is in the
hearth's travel field and, for another settlement's, within the work's walk: one lookup a field,
as today's check. Unreachable fields therefore never take a slot. If the offer, once worked out,
names a place that is not reachable (its first field cut off, another not), it is passed over, as
now, without spending a slot, and the search goes on; a hard stop at twice the budget bounds that
cost. Reachable employers of neighbouring settlements stay eligible on the same footing as the
seeker's own.

**4.2a What is kept, and how each change is seen.** Contacts are never kept: they are read afresh
at every decision, as ties move at every hearth session, hire, trade and fade, and kin change at
births, deaths, marriages and moves between households. Only the seen-ground pool is kept: per
travel field (the hearth's, or a cut-off home's own, `walking_field_of`), the reachable employers
and, for each, whether a field of it and its home are in reach. It is derived, never saved, and
reads these inputs, each with how a change is seen:

- *The travel field:* its origin cell and its revision (`home_field`'s `cell` and `rev`, the
  land's `route_revision`: crossings built or lost, worn ground, and the walking grid's
  `travel_revision`, which a flood's depths or a closed ford move). The pool keeps the pair it was
  built on and is rebuilt when either differs.
- *Which fields exist and where they lie:* fields are only appended in play (marking out) and their
  ground is fixed once marked; a load rebuilds every derived table. The pool keeps the fields'
  count; a new field rebuilds it.
- *Who works each field* (`Field::household`, which `best_job` reads, not its holder): every
  change in play (a regime's review, a let field, a forfeit, succession, a crop brought in for a
  household that left) goes through `fields_moved`, which drops the field index today. It would
  also count a new derived revision, kept by the pool; a change that missed it is caught below.
- *Employers' homes, settlements and existence* (threshing's place, the walk cap for another
  settlement's, a household founded, emptied or gone): `homes_moved`, already counted at a
  household added or taken away, a home moved or rebuilt, and a household moving settlement
  (`insert_household`, `life.rs`, `moving.rs`, `letting.rs`), as the field-site cache reads it
  (`site_for`'s stamp). The pool keeps it.
- *The day:* the pool is rebuilt each day, with the rotation's window.
- *Content* (the work's walk, the walking pace) does not change while a world lives.

Field state is not an input: sown, ripe, reaped, threshed, idle, reserved or left (`left_by`)
change daily and are read live by the offer at every decision (`reap_offer_with`), so they cannot
go stale in the pool; `keeps_ground` (`build.rs`) is building's, not read here. The pool holds no
offer, wage or hours. A kept list of this kind is what went stale in §8 25's first and fifth (the
day's asks for help, a kept home site), so: in debug builds (the simcheck profile) every read of
the pool is checked against one built afresh (`debug_assert_eq!`, as `fields_near` and `site_for`
do), and the mid-day save sweep (`settlements.rs`
`a_world_saved_at_ordinary_hours_goes_on_alike_for_four_hours_after_each`, on the home-site fix's
branch and not yet on verification: saved at five ordinary hours and compared section by section
four hours on, in spring) gains proposed runs at the harvest, the threshing and the sowing.

**4.2b The most work a decision does.** With `B = hire_search` and `C = floor(B / 2)` contact
slots, a seeker's household works out at most `min(2B, E)` offers at a decision, where `E` is the
eligible employers it reads: up to `C` contacts and `B` less the contacts weighed from seen
ground, each spending a slot (an employer with no work, or unable to pay, spends one too), plus
those passed over because the offer's place proved unreachable, which spend none, until `2B`
offers in all. At `B = 8`: at most 16 offers, 8 of them counted. Each offer is `reap_offer_with`:
up to three passes over that employer's fields. Reachability lookups (`TravelField::seconds_to`,
one table read each) are at most one per offer worked out (its place), plus, for contacts, one per
field and one for the home of each contact employer read until `C` are eligible: at worst
`Σ (fields + 1)` over the contacts, which number at most `48 × members` tied households plus the
kin households. Seen ground costs no lookup at a decision; building the pool costs one per field
within the straight line of the travel field's origin and one per employer home, once per travel
field and day or change above, shared by every household on that field.

**4.3 The budget.** `hire_search` (the first brief's `hire_seen`) is the number of distinct
employers whose offer one seeker's household works out at a decision: an attention budget, not a
limit on whom it can know. Employers are deduplicated first, so owning several fields takes one
slot. It is a content tuning value in the people profile, logged as a `NUDGE:` design choice in §9,
not a historical constant. Contacts take at most `C` slots; the rest, and any contacts leave
unused, go to seen ground. The value and the contacts' share are provisional design settings, to
be set from the scenes of §6 (remaining choices 1 and 2): the scenes start from 8 and at most
half, the top of 08-10 §5.6's and 08-04's 3–8, and are to hold across that range.

**4.4 The rotation, and why it excludes nobody for good.** This is an explicit approximation of a
search that renews. Contacts and seen ground are each listed by employer id; each list is read as a
window starting at `(k + day × w) mod n`, where `n` is the list's length, `w` the slots it has and
`k` a keyed draw on the world seed, a new purpose tag and the seeker's household id
(`Rng64::from_key`). The result is a function of saved state alone (day, ids, fields, ties, the
walking grid), so a save lives on as if never saved (ADR-0011 §4).

The guarantee is narrow. **Only while a list stands unchanged and the seeker searches every day**
does it reach every employer on it within `ceil(n / w)` days; seekers start at different places, so
on such days about `seekers × w / n` of them weigh each employer (as many seekers as employers and
four exploration slots: about four a day). Where the list changes daily (fields marked out or
changing hands, ties made and lost), `n` and every position after a change move, so an employer
can be passed over on a cycle and no bound holds; coverage then rests on the spread of seekers and
is measured (days from an employer's first offer to its first weighing), not promised. The window
is keyed on the day, not on searches made, so a seeker that searches only on some days sees only
those days' windows: one searching every `d` days steps `d × w` a time and, where `d × w` and `n`
share a factor, cycles through a part of the list only (remaining choice 4). A household-resolved
day searches once (ADR-0032 §2), and an Accelerated view keeps the day's window until midnight or a
consequential step, which re-reads the same window (ADR-0011 §4): both are daily searchers when
they seek work at all, and neither when they do not. A newcomer employer joins the seen-ground list the day its
fields are marked out; a newcomer seeker without kin or ties puts every slot to exploration. Those
figures are arithmetic, to be checked by the counts in §6. The window moves daily,
so within a day a Detailed seeker sees the same employers at each decision, as an Accelerated view
does: both modes read the same candidates from the same state.

**4.5 How a short-handed employer is found.** A raised wage reaches only those who weigh the
offer, so the loop is closed by renewed search, not by price:

- *Reaping:* the rotation brings about `seekers × w / n` new seekers a day; contacts weigh it
  sooner; a hand who has reaped there holds a tie and finds it again. The 5 % daily bid-up then
  works on those who see it.
- *Threshing:* the same search, with no wage signal at all (§1). Its place is the employer's
  home, so reachability is the home's.
- *Sowing:* the same search, with no bid-up, and hours capped by spare grain, so a short-handed
  sower may be found and still offer little. Sowing-wage timing is deferred (§8).

Each of the three is validated on its own (§6). If unfilled hours rise measurably for any of them,
vacancy word (remaining choice 3) is the next proposal, not a faster wage.

**4.6 Saves and modes.** No new saved state: a Detailed save at an ordinary hour reloads to the
same candidates, as the window depends on the day, not on decisions made; Accelerated views and the
pool are derived and set aside at midnight (ADR-0011 §4), and household-resolved days read the same
search once a day (ADR-0032 §2). It is a behaviour change: digests move once, and continuation from
the same save, and accounting, are what is checked (owner's review §5).

## 5. Performance

No limit amendment is proposed. Bench run 127 (2026-11-25, node3, c4ec5f0 with the four exact cuts,
against the M9 base) stood at **1.563×** for the 100k workload, **1.376×** for a village of 500
and **1.314×** for three of 400, past the clause (worlds with a Detailed settlement within 20 %,
household-resolved worlds and the 100k workload within 10 %: §9, "M10's bench clause"); ten of 60
**0.883×**. A local profile of that build's 100k day left `best_job` about a third of the samples,
nearly all in `reap_offer_with`.

The saving is an estimate. Offers worked out per decision would fall from every employer (some
2,000 in a settlement of 10,000 at about five to a household; not counted) to at most `2 × hire_search`
(§4.2b), beside the reachability lookups and up to 48 ties a member read for contacts. To reach
1.10× from 1.563×, a cut of 29.6 % of the day is needed: about 89 % of a one-third share, with
nothing added. A fifth of the hiring cost left, or the home-site determinism fix's own cost, would
miss it; the Detailed worlds' profile is flat, so no figure is claimed for them.

The plan, in order:

1. Establish the corrected determinism baseline first: the home-site determinism fix now in
   progress changes timings (combined candidate 74e6519d failed chain 192's telegraph-instrument and
   spring/river tests; they are to be diagnosed against a matched baseline before classifying).
2. Only after a separately approved hiring implementation, measure all four workloads of
   `tools/ci/bench-compare.sh` on node3 against that same baseline, with the same `BENCH_CPUS` and
   interleaved rounds, and once against the M9 base for the clause's ratios: village500
   (`--people 500 --days 90`, Detailed throughout), three400 (`--people 400 --groups 3 --size 1024
   --known --max --household 2 --days 60`: one Detailed settlement at Accelerated Max, two
   household-resolved), ten60 (`--seed 1 --people 60 --groups 10 --size 2048 --known --max
   --household 10 --days 40`) and gate100k (`--people 10000 --groups 10 --size 2048 --known --max
   --household 10 --days 5`, `BENCH_GATE=1`). The last two are household-resolved in every
   settlement, Detailed only where one is held (ADR-0032 §1): the 100k workload is not 100,000
   people in Detailed mode. The script runs gate100k once a side, base first; it would be run
   interleaved by hand with the same binaries.
3. Report for each: run-up (the "made in" and "lived to midnight" lines), steady state (median ms a
   day), spread (every round's figure, and p50, p95 and worst day), peak memory, responsiveness
   (p95 and worst day; the bench measures no pause latency), and the resolution mode.

Any later amendment for the two Detailed workloads would come to the owner with exact proposed
limits, the profile and these measurements, after bounded optimization, for approval; never
automatically. The 100k limit is not to be widened, and "resolved" stays the clause as written.

## 6. Acceptance checklist

Each item, once built, is tied to its commit and its evidence (test run, long run or bench).
Scenes are proposed names in `kernel/crates/civ-sim/tests/`.

- [ ] *Unconnected newcomer:* `people.rs` `a_newcomer_without_kin_or_ties_finds_field_work_by_search`
  (found within `ceil(n / w)` days, checked over keyed days, not one draw).
- [ ] *Newcomer employer:* `people.rs` `a_newly_settled_employer_short_of_hands_is_weighed_by_strangers`.
- [ ] *Changing vacancies:* `people.rs` `seekers_turn_to_a_vacancy_opened_once_another_is_covered`.
- [ ] *Several fields, one slot:* `people.rs` `an_employer_with_many_fields_takes_one_slot_of_the_search`.
- [ ] *Inaccessible nearby work:* `transit.rs`
  `fields_across_water_with_no_crossing_take_no_slot_from_reachable_work`, and the same with a
  ford closed by a flood.
- [ ] *Reachable neighbouring work:* `transit.rs`
  `a_neighbours_reachable_employer_is_reached_by_rotation_past_idle_home_employers`; the existing
  `a_hand_of_a_neighbouring_settlement_reaps_for_hire_within_the_works_walk` and
  `a_neighbours_reaping_is_found_exactly_while_its_walk_is_within_the_works_limit` stay passing.
- [ ] *Payment capacity:* `a_household_with_no_grain_to_pay_hires_no_reapers`,
  `a_household_whose_grain_only_feeds_it_to_the_harvest_hires_no_sowing` and
  `hired_sowing_pays_from_the_spare_for_the_work_the_seed_allows_and_nothing_once_it_is_gone`
  (`people.rs`) stay passing; an employer that cannot pay is counted as weighed, not hired.
- [ ] *Own production:* `people.rs` `a_seeker_whose_own_crop_stands_ripe_reaps_its_own_before_hire`.
- [ ] *Reaping, threshing and sowing separately:* `people.rs`
  `a_short_handed_reaper_beyond_the_first_window_is_found`, `..._thresher_...`, `..._sower_...`;
  `a_household_short_of_hands_at_the_harvest_hires_reapers`,
  `a_household_whose_sheaves_wait_into_the_sowing_hires_threshers` and
  `a_household_short_of_hands_at_the_sowing_hires_them` stay passing, or each change is named.
- [ ] *Kept pool:* the debug check of §4.2a on in every scene; a scene for each input (a field
  changing hands, an employer's home moved or rebuilt, a household founded and one gone, a ford
  closed by a flood, a new field) in which the pool is rebuilt and the same decision follows as
  in a world loaded at that hour; the mid-day save sweep at the harvest, threshing and sowing.
- [ ] *Search cost:* no decision works out more than `2 × hire_search` offers (counted, §4.2b).
- [ ] *Saves and modes:* `people.rs` `a_detailed_save_mid_morning_at_the_harvest_lives_on_alike`
  (saved at an ordinary hour mid-search, loaded, lived on: every section digest as the unsaved
  world's); an Accelerated midnight save and the Detailed, Accelerated and household-resolved
  transitions (`modes.rs`, `resolution.rs`); Gate B's statistics pass.
- [ ] *Accounting:* people and goods conserved, and the ledger's wage channel equal to the wages
  paid, over the long run and the scenes (`determinism.rs`, `roundtrip.rs` as they run).
- [ ] *Long run:* the dashboard worlds against a long run of the corrected determinism baseline,
  reporting by task (new report-only counters, never saved): candidates per decision (contacts,
  seen, unreachable passed over), offers worked out per decision (search cost), unfilled hours,
  hired hours, wages paid a hour, days ripe standing before reaping and the standing loss, sowing
  day against the window, food per person and hungry days, grain per hour of field work, and
  population. **Fewer hired sessions is neither success nor failure**; nothing is tuned toward a
  predetermined outcome.
- [ ] *Bench:* §5's four workloads, with `NUDGE:` entries for `hire_search`.

## 7. Remaining choices for the owner

1. **`hire_search`'s value.** A provisional design setting (`NUDGE:`), not settled: the scenes
   start from 8, the top of the research's 3–8 (08-04: agriculture needs its own policy, and
   harvest work is urgent), and the value is to be set from what they and the long run show of
   unfilled work. Lower is cheaper and leaves more work unfilled; higher costs more and finds more.
2. **Contacts' share.** Provisional in the same way: the scenes start from at most half. More
   favours remembered employers (08-04's persistence) but finds newcomers more slowly; less does
   the reverse.
3. **Vacancy word.** Recommended not in this slice: "hands wanted" reports told at the hearth as
   price reports are would close the loop faster for threshing and sowing, which have no wage
   signal, but add saved state, a schema step and likely an ADR-0019 amendment. Proposed only if
   unfilled hours rise.
4. **The rotation's step.** Recommended by day: Detailed and Accelerated agree, saves are simple;
   a window that moves each session would search faster within a harvest day at the cost of
   keying on the time of day. Against a seeker searching every few days cycling through a part of
   the list (§4.4), the offset could instead be a fresh keyed draw each day: no such cycles, but
   coverage only by chance (`1 − (1 − w / n)^searches`), with no bound at all.
5. **An offer whose place is cut off.** Recommended unchanged (passed over, no slot spent);
   choosing the first reachable field instead would change `reaping_offer` and the observer's
   `field_hire`.
6. **Neighbouring settlements known first?** Recommended no: a field within the walk is seen
   ground, and its offer is read only when weighed, as a buyer sees offers at a door (ADR-0019
   §1); requiring the settlement known (ADR-0018 §4) would hide reachable work from newcomers.

## 8. Not approved by this review

- The implementation itself, any parameter change, any performance-limit amendment, and promotion.
- Deferred separately by the owner: the price-convergence persistence proposal, the notables
  under-proposal question, sowing-wage timing, and the other ADR and sewer decisions.

## 9. As built (2026-11-26, the first candidate)

Built by Claude on `claude/bounded-hire` from verification `ffe1e161` (the corrected baseline),
under the owner's decisions of 2026-11-26, which override §4.4 and §7's choices 1 to 4.

- **Where.** `population/search.rs` (new): `field_search` and `search_field_work` (the search),
  `hire_pool` and `build_hire_pool` (the kept list), `hire_contacts`; `population/firm.rs`
  `best_job` takes the travel field's key and weighs what `field_search` finds beside the workshops,
  as before. Offers, wages and payment (`reap_offer_with` and the three under it,
  `hired_reaping`, `hired_threshing`, `hired_sowing`), workshop hiring and the observer's
  `field_hire` are unchanged.
- **Selection (replaces §4.4).** Contacts (§4.1 1, read afresh at every search) are drawn first,
  then the kept list (§4.1 2), each without replacement by a sparse Fisher-Yates draw from
  `Rng64::from_key(seed, PURPOSE_HIRE, household, day, 0 or 1)` over the list in id order. Whom a
  search weighs is therefore a function of the world seed, the household, the day and saved state
  (fields, households, ties, kin, the walking grid), so a world saved at any hour and loaded draws
  the same; no shared random stream is touched. A search never weighs an employer twice or its own
  household. No discovery deadline holds: over keyed days an employer is weighed in about
  `slots / employers` of a seeker's searches (checked in the scenes over 400 keyed days).
- **The budget.** `firm.hire_search` (8) slots and at most `firm.hire_contacts` (4) of them to
  contacts (content API 238; provisional design settings, NUDGE in plan §9). An employer weighed
  with no work, or none it can pay for, spends a slot; one whose offer's place proves out of reach
  (another settlement's beyond the work's walk) is passed over without one; a search stops at
  `2 × hire_search` (16) offers worked out, retries included.
- **The kept list (§4.2a).** Per travel field key: the seeker's settlement's households with a
  field whose centre, or a home, lies in the travel field; and other settlements' households with
  a field whose centre lies within the work's straight line of the travel field's origin
  (`fields_near`, the walk at top speed, now measured from the origin the walk is measured from
  rather than from the seeker's home) and a field or home within the work's walk. Rebuilt when its
  stamp differs: the travel field's origin cell and revision (`HomeField` `cell`, `rev`), the
  seeker's settlement, the fields' count, `field_moves` (new, counted at every `fields_moved`),
  `homes_moved`, and the day. Let go by `forget_derived`; never saved. Every kept read in debug
  builds is checked against one built afresh (`debug_assert_eq!`).
- **Differences from today's eligibility.** Own-settlement households with no field are no longer
  weighed (they offer nothing). Another settlement's household whose home, not a field, is within
  the walk is now listed (threshing's place is the home). The straight-line bound is taken from the
  travel field's origin.
- **Counts (report only, never saved).** `Population::hire_tally` (`HireTally`): searches, offers,
  slots, passed over, searches at the limit, the most offers in one search, contacts read,
  eligible and weighed, reach lookups, lists built and read; `bench_village` prints them.
- **Scenes.** `kernel/crates/civ-sim/tests/hiring.rs`, fourteen, each with its prerequisites
  asserted and shown to fail under a temporary mutation of the behaviour it protects; the suite,
  the four bench workloads, the search's counts and the labour and food readings are in plan §9
  ("Bounded hiring, the first candidate" and the measurements after it).

