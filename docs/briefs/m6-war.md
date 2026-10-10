# M6 design brief: war, conquest and fortification

**Scope.** Plan §7's M6 ("Towns & their troubles") asks for "Operational war; conquest/annexation, vassalage, secession, merger/federation" and a "Fortification kit, moved from M5 (2026-10-10, §9): enclosures as lines of sectors (ditch, bank, palisade, gates) raised by a polity's law, on the walking grid, and takings between villages that give them a purpose before war does". Its demo includes "a siege ends in annexation", and it is to prove "war with real consequences". Plan §5.5 adds that armies "are raised from real citizens under the military policy (levy, professional, mercenary firm)", are "equipped from the economy (weapons are goods) and supplied along routes", and that "Battles resolve statistically: a Lanchester-style model". This brief covers, for two or three village polities:

- what makes people of one village take from or raid another, and how the victims and their village answer;
- how a dispute escalates or is settled;
- who fights and why: households, kin, the watch, a band, a levy;
- how a band is raised, supplied and moved;
- what happens when parties meet, without a battle score;
- enclosures and sieges: entry, endurance, relief and negotiation as competing processes;
- what conquest, annexation, vassalage, secession and merger mean for the polities and laws the kernel has;
- what war leaves behind.

Plan §4.3 sets 3–8k people in 2–3 settlements for M5–M7, so a village of up to 1,000–2,700 (my arithmetic). Lived villages are smaller so far: 18 to about 170 people in the M5 worlds (README; plan §9). Everything below must work at both sizes. Sibling briefs own fire, masonry and earthworks (`m6-fire.md`), water, wells and disease (`m6-health.md`), and incident panels (`m6-observer.md`); this one needs arson, ditches, wells and those panels from them.

**What exists to build on** (checked in the code):

- **One polity per settlement.** `Polity` (`civ-agents/src/polity.rs`) has one `settlement`, a `body`, a common `stores` kept at the hearth (or under the keeper's roof by `KeepStore`), `laws` with whole histories, `versions` of the custom (each with `seized_by` for a custom taken) and `claimed` places. ADR-0013 §1 keeps polity and settlement one to one; ADR-0018 §1 builds no polity over several settlements, and its consequences forbid "a polity governing a settlement other than its own". There are ten `PolicyKind`s (common store, keep store, against taking, keep watch, curfew, amend body, repeal, claim place, agreement, build crossing) and ten `IssueKind`s. An issue opens moves and weighs toward none (ADR-0013 §5); `RuleDeliberator` scores them.
- **Force inside a polity** (`population/force.rs`, M4c slice AI step four). One who keeps the watch may go to collect a refused finding. Each adult of the household answers `Met::Yielded`, `Barred` or `Struck`, against a reluctance to strike keyed per person in `strike_threshold` [1, 4]. The watcher takes it by force if it is still worth it less `w_harm` 1 per resister. `blow()` draws `hurt_days` 3–21 and kills at `kill_share` 0.02, design priors ("no report gives a rate for a blow among villagers", plan §9). Every blow is a `Harm` in an `Encounter` (`crime.rs`), told in plain words; a death is `Cause::Violence`; the grievances are `Wrong::Forced` and `Wrong::Struck`; the tie act is `Act::Struck`. Ties keep a `fear` field that nothing writes (`ties.rs`).
- **Takings and the watch.** `take_target` (`population/crime.rs`) skips every household outside the taker's settlement. `take_goods` carries the densest food first and leaves seed; a load is `carry_kg` 20 (`content/core/people/early_farmers.toml`). The watch walks night rounds over its own settlement's homes (`population/watch.rs`) and sees only where it stands.
- **Seizure.** A revolt holds when the store's keeper, the watch and more adults stand with it than with the gathering for `hold_days` 7, and a coup when more watchers do; a founding window then weighs the old laws by `Repeal` (ADR-0017 §4–5). A polity with no office of force cannot have a coup.
- **Relations** (ADR-0020). No relation is saved; labels nothing reads. Views per person in three domains (`views.rs`, five `ViewAct`s). `Blamed` (`word.rs`) can name another polity's household, and `Wrong::Trespass` is the one wrong across the boundary. An agreement is one record and two laws (`agreements.rs`: `Clause::{Leave, Gift, Transfer}`, `AgreementState`, `Failure`). Seeking terms weighs up to 16 packages (`population/terms.rs`); payments are carried by the store's keeper (`population/payments.rs`, behaviour `Carry`); the *tributary* label gives T/Y and T/(Y − C).
- **Residence and moving** (ADR-0018). `ResidenceWhy` has nine reasons, none of them flight. The yearly move review and `refuge` (`population/moving.rs`) weigh kin, ties and food seen, and leave grievances against outsiders out. Splinter coalitions are `places.rs`'s `Coalition`; households know places only by `PlaceHow`.
- **Walking.** `NavGrid` (`civ-world/src/nav.rs`) keeps a ground factor per 8 m cell: 1 on dry land, `wading_factor` 0.25 in fordable rivers, 0 where nobody can stand. `NavGrid::with_decks` lays open crossings' decks over it, and routes are kept against a revision (`civ-land/src/crossings.rs`). A wall can be laid the same way, lowering cells instead. Top speed is 6 km/h, 5 on the flat.
- **Goods and works.** No weapon goods exist: the axe, hoe and sickle are tools, and `hunt` needs no tool or technique (`content/core/activity/hunt.toml`). `EarthKind` (`civ-land/src/earth.rs`) has platforms, pits and spoil; building parts wear and rot (ADR-0009).
- **What M5 left** (plan §9, slice AX). Nobody takes from another village, so contested claims cost nothing, and no agreement reached a gathering in a lived world. Since the walking fix, rivers too big to wade keep some neighbours apart.
- **Scale.** A year of 3,000 people in three settlements takes about 20 minutes at Max, against a budget of ten that is not met (plan §9, slice AL).

## Where the reports agree

Nine points recur across 13-02 to 13-09, 11-11 and 07-10. I treat them as invariants.

1. **War is an outcome of disputes, not a propensity.** "A valuable or threatening situation generates a dispute → people organize support → ... coercion, settlement, or fighting follows" (13-05, executive conclusion). Its frequency is "not a requirement to roll a random number for war" (13-05 §2.5). No era, rate or roll (13-02 §4.1; 13-03, executive conclusion).
2. **Raids and campaigns are different processes.** The same event can stay a crime, or become a feud, an unauthorized raid or a war, "depending on attribution and political response" (13-05, executive conclusion, §5.1). Organizer, participants and political authority are distinct (13-05 §1.2).
3. **Forces are real people taken from real work.** An army "reallocates existing people, equipment, animals, and transport capacity" (13-06, executive recommendation). No decision "should create forces merely because a power score says they exist" (13-05 §5.3), and rebellion "should never create manpower that was absent from the population" (13-03 §4).
4. **Goods are conserved.** Supply, plunder and tribute move existing stocks (13-06 §1.4; 13-08 §1.2; 13-09 §1.5). Costs come from existing systems, not "an additional arbitrary 'war cost' debit" (13-05 §2.4).
5. **No battle score.** Physical attrition, organizational defeat and the consequences of defeat are separate, and victory emerges from them (13-07, executive recommendation). Lanchester laws are benchmarks (13-07 §1.2), casualty breakpoints fail (§1.6), and defeat is not destruction (§4).
6. **Works control movement and buy time.** Preventing entry, sustaining resistance and retaining control are separate outcomes (11-11, central recommendation). There is no "palisade = ×2" (13-08 §2). A siege is competing processes, not "a wall-health bar followed by a starvation countdown" (13-08, executive recommendation; 11-11 §1.4).
7. **Conquest changes enforceable relationships, not obedience.** Military control, administrative incorporation, political acceptance and cultural change are separate (13-03, executive conclusion). "Annexation is one possible outcome, not its natural endpoint" (13-02, executive recommendation). A union is "a continuing bargain over particular powers, revenues, and obligations" (13-04, executive conclusion).
8. **Decision-makers act on beliefs.** "The kernel knows where soldiers and supplies are. Rulers should not." (13-05 §5.3). Each side estimates the other's strength itself (13-05 §1.3), and commanders' reports come late and uncertain (13-08 §7.4).
9. **Consequences outlive the war.** "Peace should remove or reduce the causes of destruction; it should not erase their consequences" (13-09, bottom line). "Occupation does not reset the city" (13-08 §6).

**What grand-strategy games teach to avoid:**

- "Arbitrary truce immunity, narrow concession menus, or a single confidence score" (13-05 §5.6, on Victoria 3).
- Soldiers fed outside the economy: Stronghold's manual "excludes soldiers from granary rations" (13-08 §7.6).
- Compliance that "naturally rises toward completion", and game growth rates (13-03 §5.6, on Hearts of Iron IV); "loyalty progression" (13-02 §5.4, on Stellaris); a single cohesion quantity for a union (13-04 §5.8, on EU4).
- Balance values read as history (13-06 §5.4, on Songs of Syx; 11-11 §5.4, on Stronghold).
- Shortcuts: "unlimited mercenary spawning, supplies appearing because territory is friendly, free animal feeding, immediate discharge teleportation, or fixed attrition unrelated to conditions" (13-06 §5.3).
- Devastation as an aggregate with balancing parameters, in place of people, inventories and buildings (13-09 §5.4, on Victoria 3).

**Where they differ, or differ from the plan:**

- **Battles.** Plan §5.5 asks for "a Lanchester-style model with technology, terrain, fortifications, morale and a commanding notable". 13-07 §1.2 finds that no constant-coefficient law fits well, and keeps them as benchmarks and tests that "should not determine battle termination". I follow the report (open question 4).
- **War as a state.** Plan §5.5 lists "war (with war goals), truce" among relation states. 13-05 §5.4 keeps sustained-war states but tracks raids, incidents and negotiations apart; ADR-0020 §1 forbids a state that causes anything. I keep war as what people and laws did, and "at war" as a label.
- **Annexation.** Plan §5.5 absorbs the occupied settlement "under the victor's constitution, or given a puppet government". 13-03 §5.1 asks that annexation "modify or terminate specific rights and obligations, not simply change a parent pointer", and that a puppet be "an actual government entity". The ADRs keep one polity per settlement, so this needs an ADR (§3).
- **Military policy.** Plan §5.5 names levy, professional and mercenary firm. 13-06 §1.1 composes service from rules and warns against a progression from levy to professional. Workshops pay wages (ADR-0006 §5), but no polity pays anyone and per-polity currencies wait for minting (README), so M6 has service by choice and service owed by law.
- **The demo.** "A siege ends in annexation" names an outcome. 13-08 §1.8 lists seven endings of a siege, and 13-02 treats annexation as one outcome among several. §3 treats this as a plot risk.
- **Scale.** Turkana stealth parties averaged 12 people and force raids 315 (13-05 §2.2). 13-06 §2.4 suggests 1–5 % of a population for sustained and 5–15 % for brief emergency mobilization: 5–15 people from a village of 100 (my arithmetic). Tactical groups are 10–40 in 13-07 §2.3 and 20–100 in 07-10 §6.1; at village scale a whole band is one group.
- **How long a grudge lasts.** 13-05 §2.4 tests a personal grievance half-life of 2–10 years; 13-09 §2.3 an event-salience half-life of 5–30 years. ADR-0016's two layers (a claim kept until settled, an activation that fades) and ADR-0020's five-year views cover both.
- **Digging.** 11-11 §2.2 gives 0.5–2 m³ of hand excavation per worker-day, with low confidence; 13-08 §3.1's 2–3 labor-days/m³ at Masada is stone work, "unsuitable as a universal construction constant". The kernel's 8 h/m³ lies inside 11-11's range (the M5 relations brief §1.8).
- **What an enclosure is.** "An enclosure is not automatically a military installation" (11-11 §1.1), and works may deter without being attacked (11-11 §4; 13-05 §4). Measures must not count works as wars.

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| Taking across the boundary | ADR-0015 incidents, beliefs and cases as now, the incident naming the taker's settlement; appended view acts | `Take` targets among homes elsewhere a member has seen |
| A band | Purpose, target settlement, organizer or law, leader, members as they came, stage with dates, what it carried and took | An organizer's call or a law; each person's choice to go |
| An encounter | M4c's `Encounter` with two sides: each person's answers by minute, every blow, who gave way, what changed hands | The people present |
| Service owed | ADR-0015 obligations of an appended kind: days of service | A law of service; each household's choice of who goes |
| An enclosure | Sectors in order (cells, ditch earthwork, palisade parts with condition, gate), its law, work done and owed | A polity's law; work given or owed |
| A gate | Open or shut, and who keeps it | The keeper's choice |
| Terms after force | ADR-0020 agreements with appended clauses | Seeking terms; each gathering |
| Jurisdiction | Each polity's settlements, each with since and how (founded, annexed, merged, seceded) | An agreement in force, or a hold kept and a law passed |
| What war leaves | Deaths with cause and encounter; an appended residence reason; grievances and views; plunder on an appended ledger channel | Existing systems |

**Derived, not saved:** "at war" and every other relation label; anyone's estimate of the other side's strength; a siege's endurance; raid incidence and the dashboard's measures. **Size:** bands and encounters are sparse; a 354 m line is 14–35 sectors of 10–25 m (my arithmetic).

### 1.1 Taking across the boundary

**Reports.**

- Appropriation needs awareness and access, and a moral filter comes before a noisy choice (04-09 §1.2, §5.2, §5.3).
- Persistent scarcity can leave people too weak to attack, while appropriable abundance draws predation even when the attacker is not poor (13-05 §1.1).
- Early farming brings "stored harvests ... and more difficulty escaping a conflict by moving" (04-09 §3.3).
- A grievance keeps victim, alleged offender, attribution confidence, responsible group, remedy and unresolved balance (13-05 §1.7).
- Victims adapt: better storage, shared watching, relocation, demands for enforcement (04-09 §5.6).

**My proposal.**

- **Widen the targets, not the motive.** `Take`'s targets add households of another settlement whose homes a member has seen (on a visit, buying, or passing within sight), within the same reach. Need, objection, perceived risk and regard stay as ADR-0015 §2 built them. A stranger's regard is near nil, so outsiders are easier to take from; that is what to measure.
- **Attribution is what witnesses know.** A witness who knows the taker blames their household (ADR-0020 §4) and writes *harms us* toward their polity (an appended `ViewAct`). One who does not know them blames nobody. Word crosses with travellers, as for trespass.
- **Victims answer as ADR-0015 §3 has them:** let it go, demand restitution, tell, report. A demand across the boundary is a visit with a purpose. Their gathering can hear a case against an outsider, but its finding binds nobody there; collecting it means going there, which is §1.2's band.
- **The issue `raided`** (appended): a household of the settlement knows its goods were taken, or one of its people struck, by people of another settlement within memory. It opens moves and weighs toward none.
- **A household's fence** (the M5 relations brief §1.8's first route) now has a reason: the takings it believes it suffered, weighed against the hours.
- **Measure first.** Before building, a probe counts how often a hungry household would rank a store elsewhere above any at home.

### 1.2 Raids: who calls one, who goes, and why

**Reports.**

- U(join raid) = E[personal spoils] + status and reciprocal benefits + avoided sanctions − E[injury and death costs] − foregone work. "A raid forms only when enough actual people join and the party can reach its target", and victims must actually lose (13-05 §1.2).
- Turkana coalitions formed without government, with sanctions against cowardice (13-05 §1.2; 13-07 §3).
- "An unauthorized raid can enrich a faction, embarrass a ruler, undermine a treaty, or force an unwanted escalation" (13-05 §1.2).
- Retaliation may target an offender, a household, a clan or a polity, and these are different escalation processes (13-05 §1.7).
- A person's response turns on "perceived compensation, coercion, communal expectations, danger, household food security, and trust in the organizer" (13-06 §1.2).

**My proposal.**

- **A raid is an episode** (ADR-0017 §3) called by an organizer. Anyone may weigh calling one at their monthly review, and at once when their household is raided (13-05 §2.4 tests 7–30 days, and reconsidering "immediately after major events").
- **Purposes are content** (appended): *take* (food from stores the organizer's people have seen), *take back* (what a finding or the household's belief says is owed) and *punish* (the household blamed). The other polity's common store, where its keeper keeps it, is a target like a household's.
- **The organizer's worth** is 13-05 §1.2's: their household's share of what they believe the band could carry home, and their grievance against the target, less the harm they expect from defenders they have seen (people, a watch, a line), the work their household would miss, the norms they hold, and any law against raiding they know.
- **Joining is each person's choice.** The organizer tells those they are tied to, and word goes by mouth. Each weighs the terms ADR-0017 §3 already scores attending by: their household's share of the spoils split among those who go, grievance, regard for the organizer, the share of those they know who go, the danger they believe, the work they would miss, and their household's food.
- **Shirking costs regard.** One who was asked and stayed home loses a little regard with those who went (13-05 §1.2's sanctions; a design prior, measured before tuned).
- **Muster is who comes.** The band is whoever is at the hearth at the hour, never the promised count (13-06 §1.2).
- **A band by law** is an appended policy kind: its purpose, the target polity, a leader, and whether households owe service (§1.4). Without such a law, raids stay possible. A law against raiding, a wrong its own gathering hears, is the polity's own answer if its people choose it.
- **Nobody raids a place nobody in the band has seen.**

### 1.3 How a dispute escalates or is settled

**Reports.**

- The loop is trigger, options, consequences, authority and participation, demands, execution, updated beliefs. Options include "doing nothing, adaptation, migration, compensation, arbitration, trade concessions, tribute, mobilization, a limited raid, defensive preparation, and sustained war"; an actor "should not choose between only 'war' and 'accept humiliation'" (13-05 §5.2).
- A peaceful split exists because fighting is costly: p − c_A/V ≤ x ≤ p + c_B/V. It vanishes when beliefs differ, and "actors can benefit from exaggerating strength or resolve" (13-05 §1.3). A shift in power can make a promise unsafe (§1.4).
- A settlement specifies allocation and implementation, and "a signatory must be able to bind the relevant participants" (13-05 §1.9). An alliance is a conditional contract: trigger, obligation, timing, exceptions, enforcement (§1.8).
- A truce of 0.25–5 years is a negotiable term whose "expiration opens reconsideration rather than automatically causing war" (13-05 §2.4).
- Separate capability from posture: "A fort may simultaneously improve defense and worsen relations" (13-05 §1.5).

**My proposal.**

- **No ladder.** Each step is a move someone holding an issue may weigh: seek terms, keep a watch, enclose, send a band, or nothing. `raided` opens them all and weighs toward none (ADR-0013 §5).
- **Clauses are appended** to ADR-0020 §6's code list, each performed by people:
  - *compensation*: a gift tied to named takings;
  - *takings heard*: each gathering hears its people's takings from the other's households, which only a polity with a law against taking can perform (the M5 relations brief §1.4's fourth clause);
  - *no raids*: each polity holds a raid on the other a wrong its own gathering hears;
  - *truce*: a term in which neither side may propose a band law against the other;
  - *assistance*: a band when the other is raided, by trigger and term (13-05 §1.8);
  - *tribute for peace*: ADR-0020's transfer, granting nothing but the other's *no raids*.
- **Strength is believed, not read.** Two claim kinds are appended (ADR-0016 §3): *band seen* (where, how many, which way) and *raid* (who, what was taken, who was struck). A negotiator's estimate of the other's strength is the people of it they and their companions have seen, faded by age. Nobody reads the other's numbers.
- **Alarm is a forecast.** A household's expectation of being raided reads the raids and bands it heard of, not palisades: a wall is capability, a band on the move is posture (13-05 §1.5).
- **Threat enters forecasts, never as a state.** A household weighing tribute for peace sets the transfer against the raids it expects if it refuses (13-02 §4.2: EU(resist) − EU(comply)).

### 1.4 Who fights, and service owed

**Reports.**

- Keep five quantities apart: service-liable, mustered, deployable, field combatants and total supported (13-06, executive recommendation).
- A service obligation names "issuer, liable_entity, eligibility_rule, required_role, equipment_provider, compensation, start_condition, duration, geographic_scope, exemptions, discharge_conditions" (13-06 §1.1). Household feasibility, physical muster and finite service matter (§1.2).
- Ask "who serves, for how long, where, under whose command, with whose equipment, and at whose expense?" (13-02 §1.2 E). A summons can fail "because service expires, not only because 'loyalty' is low" (13-02 §4.4).
- Local defence and distant campaigning have different participation limits (13-06 §4.2).
- Fighting skill draws on civilian life (07-10 §1.3); 40–120 supervised hours for basic spear-and-shield handling is a design prior with low confidence (07-10 §2.2).
- Leadership is information, order delay, coordination, reserves, succession and trust, never a bonus (13-07 §1.8).

**My proposal.**

- **Four ways to fight, each chosen:** defending one's own home and store (M4c's answers); coming to a neighbour's aid when one sees or hears it (regard for them, grievance against the takers, danger); the watch confronting outsiders on its rounds; and going with a band.
- **A law of service** (an appended policy kind). The polity issues it; each household owes days of service per eligible adult a year; households equip their own; nobody is paid; a band law starts it; the band's target bounds it; the band's return or the days' end discharges it. 13-03 §2.3 tests 0, 15, 30 and 60 days. It is owed as an ADR-0015 obligation of an appended kind. The household chooses who goes, and may keep it back as a levy is kept back (M4c's refusal).
- **Eligibility is by age, from the template.** No rule by sex is authored; who goes is each household's choice.
- **The leader** is a person whose word to stay or give way members weigh by their regard for them.
- **Skill comes by practice** (ADR-0006 §2): fighting is a skill gained by fighting, and by drill only if someone chooses to drill (open question 7).

### 1.5 Raising, supplying and moving a band

**Reports.**

- Mobilization runs "notified → preparing → traveling to muster → inspected → training/organizing → assigned → deployed → discharged → returned" (13-06 §1.2).
- Supply conserves goods; food has a preparation state; purchase, taxation, requisition and plunder differ; "Foraging must draw from something real" (13-06 §1.4).
- What matters is "whole-force progress, not the walking speed of the first person" (13-06 §1.5).
- "Avoid a universal modifier such as 'mobilization reduces production by 20%.'" Remove actual hours from actual tasks; sowing and harvest differ (13-06 §1.6).
- Test settings: a personal food reserve of 3–10 days, and a foot column at 15–25 km a travel day (13-06 §2.4). Campaign seasons emerge from intersecting calendars, and stocks can finance armies without a cash treasury (13-06 §3).

**My proposal.**

- **Stages:** called, gathered, out, camped, coming home, home, dispersed, each dated.
- **Supply is what members carry.** Each takes from their household's store the days of food the household chooses, within `carry_kg`, leaving room for what they hope to bring back. Bread spoils in days and grain must be ground before it is eaten (plan §9, slice H), so a long absence needs provisions, dried fish or meat, or what is taken. A band law may give the band goods from the common store, set aside and carried as payments are.
- **Movement is on the walking grid,** the band at its slowest member's pace. Neighbours about 3 km apart are under an hour's walk at 5 km/h (my arithmetic), so a raid is an afternoon, and supply binds only in a siege.
- **The work lost at home is the work members do not do.** Nothing else charges for it.
- **A check, not a quota:** 5–15 % of a village of 100 is 5–15 people, and of 1,000 is 50–150 (my arithmetic on 13-06 §2.4).

### 1.6 When parties meet

**Reports.**

- "Contact and exposure → injuries and disruption → decisions and cohesion changes → withdrawal or collapse → pursuit, capture, and medical outcomes" (13-07, executive recommendation).
- E = min(N_available, K_usable), a hazard per active attacker, integer casualties drawn binomially, and both sides computed "from the same pre-step state" (13-07 §1.5).
- Morale, cohesion, suppression and fatigue are separate. Avoid "the unit always breaks at 30% casualties"; a break hazard reads recent local losses, neighbours' flight and isolation (13-07 §1.6). Withdrawal, rout and surrender differ, and pursuit needs reach (§1.7).
- Attacks on settlements need "explicit civilian presence, refuge, evacuation, and captive-taking processes. A massacre should not be represented merely as an exceptionally high army attrition roll" (13-07 §3).
- P(death) = P(violent encounter) × P(dangerous injury | encounter) × P(death | injury) (04-09 §5.5).
- Basic hafted weapons are starting capabilities, not discoveries after farming (07-10 §3.1, W01). "Battles should not routinely consume every participant" (07-10 §5).

**My proposal.**

- **An encounter is M4c's, with two sides.** Everyone present takes part by their own choice: the band, the household, neighbours who came, the watch. Each chooses each minute (the kernel's minute, at the top of 13-07 §2.3's 10–60 s range): press on, stand, give way, flee, or let goods be taken.
- **The place limits contact.** Pairs form up to a frontage K: a doorway, a gate's width, open ground. Each pair exchanges blows at a per-minute hazard from what each holds, and each blow is M4c's `blow()`: days kept from work, a death at `kill_share`. Most encounters will be lumpy or bloodless.
- **Staying is a choice.** A person's will to stay falls with blows they saw near them in the last few minutes (13-07 §2.3 tests 2–10), with others of their side giving way, and with their own hurts. There is no casualty threshold.
- **Ending and pursuit.** It ends when one side has nobody pressing on. Only those who choose to, and can reach, pursue.
- **Those who do not fight** (children, the old, anyone who chooses not to) flee to kin, to the hearth or out of sight, or stay. Harm to someone not fighting is a striker's choice, recorded as such (06-10 §4.C), never spread by formula.
- **Equipment** (open question 6). v0 adds a wooden spear made with wood shaping, beside the axes households hold. What a person holds sets their hazard and reach, never a damage multiplier.
- **Lanchester laws are tests.** A gate-limited exchange keeps a linear invariant, and more people behind the gate add no blows (13-07 §1.2, §4).

### 1.7 Enclosures and sieges

**Reports.**

- Palisades block and slow; ditches add a descent, an exposed crossing and an ascent; ditch soil can build the rampart, and digging it is not charged twice (11-11 §1.2). Progress is the least of labour, materials, tools and work front; incomplete works give partial benefit; "compulsory labor is not economically free" (§1.3). Upkeep is work orders (§1.5).
- Build when perceived avoided losses, access control and political benefits exceed construction, upkeep and opportunity costs, as particular decision-makers judge them, among other responses (11-11 §1.1).
- Priors: 0.5–2 m³ of digging a worker-day; 3–4 people per post kept day and night; sectors of 10–25 m; timber replacement tested at 5, 15 and 30 years (11-11 §2.2, low confidence except the post arithmetic). A 100 m ditch is 500 m³ and 250–1,000 worker-days; 100 m of close-set palisade is about 500 posts (§2.3).
- A siege turns on access, sustainability, local military conditions and willingness to continue (13-08, executive recommendation). "A force outside one gate does not automatically blockade the whole settlement" (§1.1). Water is a separate balance (§1.2). The besieger must also survive, and "More soldiers need not mean faster success" (§1.3). "A breach creates an opportunity, not automatic capture" (§1.4). Defence is active (§1.6).
- Endings: negotiated capitulation, capture by assault, betrayal, breakout, relief, attacker abandonment, political settlement. "Fear of an untrustworthy victor can make resistance stronger, not weaker" (13-08 §1.8).
- F/(Q − I) "is not a surrender timer", and protecting the interior differs from protecting its fields (11-11 §1.4). Do not infer siege machinery from a settlement ditch; "Where relocation is cheap, flight can compete with prolonged defense" (13-08 §4).

**My proposal.**

- **The enclosure as M5 designed it** (plan §9, "The fortification kit moves to M6"): a line of sectors 10–25 m long. Each may have a ditch (an ADR-0010 earthwork; ditch and bank appended to `EarthKind`), its spoil a bank inside; a palisade of ADR-0009 parts rotting at the foot by wetness; and gates.
- **It is laid on the walking grid as decks are,** lowering cells: a sector's cells at the least of the ground and a climbing factor (slow, and seen by anyone keeping watch within sight); a gate's cells as the ground while open, as the sector while shut. Routes are kept against its revision.
- **It is raised by law.** The law names the line (the store, the hearth, or homes within a reach), how work is raised (days owed, or volunteers) and whether the watch keeps the gates. Each household weighs the takings and raids it believes it suffered and expects, its share of the work, the walk round to a gate, and whether its home is inside. Households outside pay without cover.
- **Illustration** (the M5 relations brief's arithmetic, carried): a 1 ha core needs at least 354 m of line, about 1,770 posts and about 14,200 hours of ditch at 8 h/m³.
- **A siege is not a mode.** It is a band camped near a shut line, taking what lies outside (fields at harvest, homes outside the line) and stopping those it sees leave. Camping overnight is new: presence apart from residence, one of ADR-0018's revisit triggers.
- **Endurance is stores and water.** Inside are the households' stores and the common store. No village has a well yet, so every walk to water goes out through a gate (13-08 §1.2); a well inside the line, if the health brief's wells come, changes that. The band's endurance is what it carried and takes, and its members' work at home: the harvest calls them back.
- **Entry is local:** a gate opened from inside (a revolt or coup of those who would rather take terms), a rotted or cut gap, climbing at night, or an assault where the frontage is a gate's width.
- **Each of 13-08 §1.8's endings has a mechanism:** terms (an agreement each gathering ratifies; the besieged can meet inside); capture (this brief's §1.8); betrayal (a gate opened); breakout (households leaving by a route the band does not watch); relief (an *assistance* clause); abandonment (members go home one by one, never all at once on a date).

### 1.8 Conquest, annexation and vassalage

**Reports.**

- Direct annexation, indirect rule, a client government, military occupation and colonization allocate powers differently, kept as separate fields: appointments, taxation, revenue retention, courts, recruitment, religious administration, land allocation and external relations (13-03 §1.1).
- Conquest "transfers usable administrative capacity only where personnel, records, and organizations survive and cooperate", and annexing must not "instantly reveal every household's wealth" (13-03 §1.2). Compliance, evasion and rebellion diverge: "A quiet settlement is therefore not necessarily a loyal settlement" (§1.6).
- A superior compares leaving a neighbour independent, an unequal relationship and direct rule; "Do not let annexation become automatically cheaper merely because the subject has been dependent for many years" (13-02 §1.2 A). The ruler's bargain is not the households' (B). Annexation needs both the wish and the capacity, and administrative replacement must actually happen (§4.3).
- A surrender is a contract over military personnel, political authority, property, civilian safety and enforcement, and reputation updates "from the actual treatment of people, not merely from the signed agreement" (13-08 §5.2).

**My proposal.**

- **Military control is a hold, and changes nothing by itself.** A band holds a settlement when its members keep the hearth and the store for `hold_days` (7) and nobody of the settlement stands against them: ADR-0017 §4's test, applied from outside.
- **What follows is a law of the victor's own gathering,** decided by its custom with every stance kept: leave, demand terms (an agreement the held polity's gathering must still ratify), or annex.
- **Annexation closes the held polity's deciding.** Its record is kept; its last custom version names the polity that took it; its residents become members of the victor's polity. A founding window (ADR-0017 §5) lets the victor's body weigh each of the old laws. The old common store stays where it was, now the victor's, on a ledger channel that records how it changed hands.
- **The same change by agreement** is an appended *one body* clause: a surrender or a merger, ratified by both gatherings.
- **Afterwards compliance is each person's choice:** levies paid or kept back at threshing, gatherings attended or not, petitions, refusals, revolts. The victor's people know the annexed households' stores only as they knew them before.
- **A polity over several settlements** keeps a list of settlements, each with since and how. Membership is residence in any of them. The body gathers at its seat's hearth, so people of the far village walk there or are absent (13-04 §1.7's deadlock and domination, arising from distance). The watch keeps rounds where its holder lives. This is the expensive part (§3).
- **Vassalage in v0** is a bundle of existing and appended clauses: a transfer, *assistance* and *no raids*. The *tributary* label and its burden ratios exist. Decision-right clauses (appointments, other agreements only by leave) and client governments wait (§2).

### 1.9 Secession and merger

**Reports.**

- A movement's objective may be a policy, replacing officials, the centre, autonomy or independence; "Do not make independence the default endpoint of regional anger." Secession needs a coordinated constituency, a recognizable unit, a viable arrangement, a parent unwilling or unable to keep it, and an outside environment; de facto independence, acceptance and recognition are separate (09-13 §1.2).
- Formation requires "two bargains: between polities and within them" (13-04 §1.2); powers are negotiated by domain (§1.3); "a common budget creates claims, not resources" (§1.4).
- "A union should not initially delete its constituent polity IDs" (13-04 §5.1); withdrawal settles debts, common assets and obligations (§5.5).

**My proposal.**

- **Secession is a revolt scoped to one settlement** of a polity of several (ADR-0017 §4). The faction's program is a body of that settlement's residents. It holds when the store kept there and the watch there stand with it, and more adults there stand with it than with the polity's body, for `hold_days`. The new polity founds with a copy of the body, as a daughter does (ADR-0018 §1). The parent answers with its own moves. Splinter founding stays secession's other form, by leaving (plan §5.5).
- **Merger** is the *one body* clause between two polities that both ratify it, naming whose custom governs, or a gathering of both. Settlements, records and laws are kept, and each old polity's laws are weighed in a founding window.
- **Federation's competence matrix waits** (§2). The *assistance* clause is v0's league.

### 1.10 What war leaves behind

**Reports.**

- Keep two classifications: status when affected (combatant, civilian, captive, displaced) and outcome and cause. "Capture transfers control over a living person; it does not remove that person from world population" (13-09 §1.1).
- Displacement is repeated and household-level, by safety, food access, livelihood and kin support against journey, housing and exclusion; "Refugees should not carry an intrinsic productivity or instability penalty" (13-09 §1.4).
- Destroyed, captured and inaccessible capital differ; for early farmers, seed grain, food reserves, tools and labour at planting and harvest come first (13-09 §1.5).
- Exposure can raise local cooperation while grievances against particular groups remain (13-09 §1.8).
- Priors: safe return for 5–30 % of eligible displaced households a year; an agrarian restart in 1–3 harvests; personal event-salience half-lives of 5–30 years (13-09 §2.3, low confidence).

**My proposal.**

- **Deaths** keep `Cause::Violence` and their encounter. The encounter records whether each person was fighting, so combatant deaths and others are told apart. Wounds keep their days; lasting impairment waits.
- **Flight.** A household raided, or that has heard of raids near it, reviews where it lives at once (ADR-0018 §5's event review), with safety among the terms. A residence reason, *fled*, is appended. Return is a later review of the same kind. A household that fled keeps its claim to its ground (13-09 §1.4 keeps former home and property claims apart from location). Today a household that moves ends and is made anew elsewhere (`relocate_as`), so its holdings pass by ADR-0007 §2's succession.
- **Plunder** moves on an appended ledger channel. An emptied store, seed taken, a sowing missed: the economy carries the rest (13-09 §1.5). Whether raiders take seed is a switch to measure with 13-09 §4's test of a raid before planting against one after harvest.
- **Grievances and views.** Those raided hold grievances against the raiders' households (appended `Wrong`s) and views that the other polity *harms us*. Where a band was sent by law, they also blame its body, which extends ADR-0020 §4. A broken *no raids* clause counts against *keeps its word*.
- **Matched runs.** `civ-host twin` lives a save twice; its war-free twin gives 13-09 §4's "matched war and no-war runs".

## 2. What to defer, and why

| Defer | To | Why |
|---|---|---|
| Captives, hostages, enslavement, forced relocation | M7 or later | Distinct statuses (13-03 §1.9) and coercive relationships kept apart from the rest of a person (06-10 §4.A) need custody and household membership the kernel lacks |
| Professional forces, hired bands, soldiers' pay, war debt | M7 | No polity pays anyone yet, and credit is M7's (plan §7); debt is a liability with a creditor (13-09 §1.7) |
| Siege engines, mining, masonry walls | M8 or later | "Do not infer sophisticated siege machinery from the existence of a settlement ditch" (13-08 §4); they need masonry and heavy carpentry (07-10 §3.3, F02–F03) |
| Bows, shields and armour | With weapon content (open question 6) | Each needs its own materials and skills (07-10 §3.1, W02–W03; §1.4) |
| Horses and mounted war | After riding and draught animals exist | Content has none; animals can dominate transport demand (13-06 §4.1) |
| Arson by raiders | M6's fire slice | Plan §5.4 names arson among ignitions; destruction is apart from seizure (13-09 §1.5) |
| Client governments, decision-right clauses | M7 | Appointments and external relations as separate powers (13-03 §5.1; 13-02 §5.1) |
| Federation's competence matrix; three-party agreements | M7 | 13-04 §5.2's eight domains; two or three settlements need little beyond two parties |
| Assimilation of language and religion | M9 or later | No language or religion systems; 13-03 §1.8 keeps them separate processes |
| Disease of camps and refugees | With M6's disease slice | War changes exposure; the disease model kills (13-09 §1.2) |
| Emblems and banners | After bands work | The M5 relations brief §1.9 put them with M6's war bands |
| A war god tool | Never | It would choose an outcome; M6's drought tool (plan §2) can make lean years to test what follows |

## 3. Risks and validation

**Plot.** The demo names its outcome. A siege that ends in annexation must come from who was hungry, who went, what each gathering decided and what the besieged believed of the victor (13-08 §1.8). After a week without one, nudge a prior (a raider's valuation of a store they saw, or how long members stay before their harvest calls), never an event, and log a `NUDGE:` (plan §1, rule 3). No date, era or size may trigger a raid, and no issue weighs toward a band.

**Nothing to fight over, or too much.** M5 found that villages share ground at no cost, land is plentiful, and rivers now keep some neighbours apart (plan §9). Raids may never form. Or they may form whenever a household is hungry, against "Scarcity should not automatically spawn an army" (13-05 §1.1). Count the funnel before tuning anything: targets elsewhere ranked, takings across the boundary, raids weighed, called, joined and mustered, encounters, blows and deaths, issues held, terms sought, laws passed. A move never invoked may be "an unreachable action", not a preference (02-04 §2). Flight may beat defence where moving is cheap (13-08 §4); that is a result, not a failure.

**Leaking truth.** No choice reads the other side's numbers, stores, laws or bands except through claims (13-05 §5.3; ADR-0020 §2). Test that a band nobody saw changes nobody's forecast, and that a negotiator's estimate of the other's strength traces to sightings.

**Compute.** War must cost nothing at peace: a world with no outsider in reach gives the same digests as the build before, as every M5 step was checked. Widening `Take`'s targets is the one cost every hungry household pays; limit it to homes members have seen, and cache it as slice AL's searches are (plan §9). Minute-by-minute encounters cost only while people are in them, and a siege adds a daily review for the band's members. Measure a year of the M5a demo's world at Max with force and as its twin without, against the 20 minutes already over a 10-minute budget. Add bands, encounters and deaths by violence to Gate B (ADR-0011 §5), with wide tolerances.

**Stylized facts and tests** (plausibility checks, never targets):

- **Lumpy small fights.** 50 exposed people at 2 % each see no casualty about 36 % of the time (13-07 §4: 0.98^50 ≈ 0.364). Most village encounters should end with few or no deaths.
- **Raid deaths.** Turkana force raids killed 1.1 % of participants a raid, 1.3 % when combat occurred (13-07 §2.1; moderate confidence, with firearms). Greek battle winners lost about 5 % dead and losers about 14 % (low to moderate). A village band losing a large share every raid is wrong.
- **Local catastrophe beside modest averages.** Ōhaeawai's assault party lost 44 % (40 killed and 70 wounded of 250) while others were not committed (13-07 §2.1, §4). An assault at a gate may be costly without the whole band being so.
- **Usable contact.** People added behind a gate add no blows at once (13-07 §4), and both sides strike from the same pre-step state (§1.5).
- **Different scales.** Stealth parties averaged 12 and force raids 315 (13-05 §2.2): bands should vary with who chooses to go, not a fixed share.
- **Siege length is not wall strength.** Hold the line fixed and vary stores, water, leakage and the band's harvest; outcomes should diverge (13-08 §6). Brosset's "model siege" ended within 45 days and Vicksburg lasted 47 (13-08 §3.2; neither is a mean). The arithmetic 90/(1 − s) days (13-08 §3.4) checks the stores and never sets a date.
- **Negative predictions.** A band that cannot feed itself goes home, and a village whose gate nobody watches is not starved (13-08 §6).
- **Defeat is not destruction.** A held settlement keeps its people, ties and grievances (13-07 §4; 13-08 §6).
- **Bargaining unit test.** With shared beliefs and costly fighting, some split satisfies p − c_A/V ≤ x ≤ p + c_B/V; with divergent beliefs it can vanish (13-05 §1.3: "a useful unit test").
- **Recovery takes time.** A 20 % loss at 1 % growth takes 22.4 years to regain (13-09 §2.2); a depopulated village should not refill in a few years.
- **Twin runs** of a raid before planting against one after harvest, and of the only mill against an ordinary workshop (13-09 §4).
- **Deterrence counts.** A line never attacked may still have worked (11-11 §4); report works and raids apart.
- **Measures, reported and not graded** (13-05 §2.1): raids per reachable polity pair a year, war episodes, deaths per 1,000 person-years by cause, peace durability.

**Observer.**

- Bands walk visibly on the map (plan §5.5); lines show each sector's condition and each gate open or shut. Each polity's settlements are listed with how and when each came.
- An encounter is told in plain verbs, with who struck whom and under which law or call (06-10 §4.C–D; ADR-0017 §6), in the Order panel and the chronicle. Kernel truth and what each side believes stay apart.
- A siege panel shows each side's food at current use, what still comes in and the band's endurance, never "siege progress: 63%" (13-08 §7.5). Every agreement made after force shows both sides' law histories.

**Choices that are expensive to reverse.** Plan §1 allows about three ADRs a milestone, and fire, water and disease will want theirs. I suggest one ADR for force between polities: bands and two-sided encounters; service as an obligation kind; appended clauses, claim kinds, view acts, wrongs, a residence reason and a ledger channel; and a polity over several settlements, amending ADR-0013 §1, ADR-0018 §1 and ADR-0020 §4's rule on blaming a body. Enclosures amend ADR-0009 and ADR-0010, as the M5c design planned. A suggested order: takings across the boundary and enclosures first (the plan's "before war does"), then bands and encounters, then sieges, then changes of sovereignty, then the demo.

## 4. Open questions for the designer

1. **Unauthorized raids.** May anyone call a raid, as 13-05 §1.2 allows, or only a law? Without the first, a treaty cannot be undermined by its own side's raiders.
2. **One polity over several settlements.** Is annexation a change of membership, as proposed, or only tribute and agreements in M6? The first needs the ADR.
3. **Where a body of two settlements gathers:** at one seat, as proposed, or at each hearth by turns?
4. **Battles.** Plan §5.5 asks for a Lanchester-style model; 13-07 keeps it only as a test. Are per-minute choices and blows acceptable in its place?
5. **Captives.** Leave capture out of M6, as proposed, though early farming massacre evidence (13-07 §3; 13-09 §2.1) shows whole communities attacked?
6. **Weapons.** Author a wooden spear now and bows and shields later? Hunting has no tool today.
7. **Drill.** Is fighting skill gained only by fighting, or may a law ask for drill (07-10 §2.2's hours)?
8. **Tribute under threat.** Is a threat only a forecast term, as proposed, or may a band's leader state a demand that travels as a claim?
9. **Seed.** May raiders take seed grain (13-09 §1.5), or does `take_goods`'s rule hold?
10. **The demo's nudge.** Which prior, if any, may be nudged after the time box?
11. **Federation.** Are the *one body* and *assistance* clauses enough for M6's "merger/federation"?
12. **Safety when moving.** Should the yearly move review weigh raids heard of, as proposed, or only the review an actual raid prompts?
