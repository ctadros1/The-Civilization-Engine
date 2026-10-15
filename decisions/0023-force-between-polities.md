# ADR-0023: Force between polities: takings across the boundary, bands, encounters, enclosures, and a polity over several settlements

Status: Accepted
Date: 2026-10-15
Milestone: M6c

## Context

Through M6b, force exists only inside a polity. One who keeps the watch may go to collect a
refused finding. Each adult of the household answers it, and every blow is `blow()`'s
(`population/force.rs`, ADR-0017 §6): `hurt_days` 3–21, a death at `kill_share` 0.02, both
design priors. Takings stay within a settlement: `take_target` skips every household outside the
taker's (ADR-0020 §4: "walking to another village's stores to take is a raid, and waits for
force between polities"). The watch walks only its own settlement's homes. No weapon goods exist.

Polity and settlement are one to one. ADR-0013 §1 says so, and ADR-0018 §1 builds no polity
over several settlements. Its consequences forbid "a polity governing a settlement other than
its own".

M6b left fire, masonry, banks and ditches physical:

- banks and ditches are lines of segments (`EarthKind::Bank` and `Ditch`, slice BG);
- walls can be rubble or brick;
- buildings burn as parts;
- a revolt or coup holds by ADR-0017 §4's test over `hold_days`.

M5 found that nothing makes contested ground costly. No agreement reached a gathering in a
lived world, and nobody takes from another village (plan §9, slice AX).

M6c's plan text asks for:

- takings and raids between villages;
- operational war: who fights and why, a band raised, supplied and moved, fighting and its
  losses;
- the fortification kit carried from M5;
- sieges;
- conquest and annexation, vassalage, secession, merger and federation;
- a demo: "a siege ends in annexation".

The war brief (`docs/briefs/m6-war.md`) digests 13-02 to 13-09, 11-11 and 07-10. Its nine
invariants are:

1. war is an outcome of disputes, not a propensity;
2. raids and campaigns are different processes;
3. forces are real people taken from real work;
4. goods are conserved;
5. there is no battle score;
6. works control movement and buy time;
7. conquest changes enforceable relationships, not obedience;
8. decision-makers act on beliefs;
9. consequences outlive the war.

13-07 §1.2, §1.5–1.7, §2.3 and §5, and 13-05 §1.2–1.3, were checked for this ADR. Two
quotations carry the most weight:

- On Lanchester laws, 13-07 §1.2: they should not "determine battle termination".
- On contact, §1.5: E = min(N_available, K_usable), with both sides computed "from the same
  pre-step state".

This is expensive to reverse because it adds:

- a saved band record and an encounter with two sides;
- service as an obligation kind;
- enclosures on the walking grid;
- appended clauses, claim kinds, view acts, wrongs, a residence reason, issues, policies and a
  ledger channel;
- a polity's list of settlements, which changes what membership means in every system that
  reads it;
- a save schema bump.

## Decision

### 1. No war calendar, and no war state

- **Disputes make war.** No date, era, rate, size, prosperity or time since the last raid makes
  a raid likelier (13-05 §2.5; 13-02 §4.1).
- **Issues weigh toward nothing.** An issue opens moves and weighs toward none (ADR-0013 §5).
  No issue weighs toward a band.
- **"At war" is a label.** As ADR-0020 §1 has it, it describes what people and laws did and is
  never saved. Nothing reads it.
- **Counts are outputs.** Raids, encounters, blows, deaths, sieges and changes of sovereignty
  are counted and reported, never targeted. Nothing reads a count to steer.
- **War costs nothing at peace.** A world where nobody has seen a home elsewhere gives the same
  digests as the build before this ADR.

### 2. Taking crosses the boundary where people have been (amends ADR-0020 §4)

- **Targets.** `Take`'s targets add households of another settlement whose homes a member of the
  taker's household has seen: on a visit, a purchase, a carry, or passing within sight. The
  target must be within the walking reach the household already uses for trips. Need, objection,
  perceived risk and regard stay as ADR-0015 §2 built them. A stranger's regard is near nil, so
  outsiders are easier to take from; that is measured, not tuned away.
- **Attribution is what witnesses know.** A witness who knows the taker blames their household
  and writes *harms us* toward their polity (`ViewAct` appended). One who does not know them
  blames nobody they can name. Word crosses with travellers (ADR-0020 §2).
- **Victims answer as ADR-0015 §3 has them:** let it go, demand restitution, tell, report.
  - A demand across the boundary is a visit with that purpose.
  - Their gathering may hear a case against an outsider, but its finding binds nobody there.
  - Collecting it means going there with others, which is §3's band.
- **The issue `raided`** (appended) is held by a household that knows its goods were taken, or
  one of its people struck, by people of another settlement within the memory window.
- **Probe first.** Before targets widen, a probe counts how often a hungry household would rank
  a store elsewhere above any at home.

### 3. A band is an episode with an organizer, and its members came by choice

- **Who may call one.** Anyone may weigh calling a band at their monthly review, and at once
  when their household is raided (13-05 §2.4 tests 7–30 days and reconsidering "immediately
  after major events"). A law may call one too (§5). Unauthorized raids are allowed: without
  them, a treaty cannot be undermined by its own side's raiders (13-05 §1.2).
- **Purposes are content** (append-only):
  - *take*: food from stores the organizer's people have seen;
  - *take back*: what a finding or the household's belief says is owed;
  - *punish*: the household blamed.

  The other polity's common store, where its keeper keeps it, is a target like a household's.
- **The organizer's worth** follows 13-05 §1.2. In favour:
  - their household's share of what they believe the band could carry home;
  - their grievance against the target.

  Against:
  - the harm they expect from the defenders they have seen;
  - the work their household would miss;
  - the norms they hold;
  - any law against raiding they know.
- **Nobody raids a place nobody in the band has seen.**
- **Joining is each person's choice.** The organizer tells those they are tied to, and word goes
  by mouth. Each weighs:
  - their household's share of the spoils, split among those who go;
  - their grievance;
  - their regard for the organizer;
  - the share of those they know who go;
  - the danger they believe;
  - the work they would miss;
  - their household's food.

  Eligibility is by age from the template, never by sex.
- **Shirking costs regard.** One who was asked and stayed home loses a little regard with those
  who went (a design prior, measured before it is tuned).
- **Muster is who comes.** The band is whoever is at the hearth at the hour, never the promised
  count (13-06 §1.2).
- **Stages are saved and dated:** called, gathered, out, camped, coming home, home, dispersed.
- **Supply is what members carry.** Each member takes the days of food their household chooses
  from its store, within `carry_kg`. A band law may also set goods aside from the common store,
  carried as payments are.
- **Movement is on the walking grid** at the slowest member's pace.
- **Work lost is work not done.** The work lost at home is the work members do not do; nothing
  else charges for it (13-06 §1.6).
- **The record** keeps:
  - purpose, target, organizer or law, leader;
  - members in the order they came;
  - stage and its dates;
  - what was carried, taken and lost.

### 4. When parties meet: two sides, by the minute, with no battle score

- **Two sides.** An encounter is M4c's `Encounter` with two sides. Everyone present takes part
  by their own choice: the band, the household, neighbours who came, the watch.
- **Each minute, each person chooses** one of: press on, stand, give way, flee, or let goods be
  taken. The kernel minute sits at the top of 13-07 §2.3's 10–60 s range.
- **Contact is bounded by the place.** Pairs form up to a frontage K: a doorway, a gate's width,
  open ground (content). Each pair exchanges blows at a per-minute hazard set by what each
  holds. Both sides are computed from the same pre-step state (13-07 §1.5).
- **Every blow is `blow()`'s.** It keeps someone from work for days, or kills at `kill_share`.
  Integer harms are drawn per person, so most village encounters are lumpy or bloodless.
- **Staying is a choice, not a threshold.** A person's will to stay falls with:
  - blows they saw near them in the last few minutes (13-07 §2.3 tests 2–10);
  - others of their side giving way;
  - their own hurts;
  - their regard for the leader's word.

  There is no casualty breakpoint (13-07 §1.6).
- **Ending and pursuit.** An encounter ends when one side has nobody pressing on. Only those who
  choose to, and can reach, pursue.
- **Those who do not fight** (children, the old, anyone who chooses not to) flee to kin, to the
  hearth or out of sight, or stay. Harm to someone not fighting is a striker's choice, recorded
  as such (06-10 §4.C), never spread by formula.
- **Equipment is goods.** v0 adds a wooden spear, made with wood shaping, beside the axes
  households hold. What a person holds sets their hazard and reach, never a damage multiplier.
  Bows, shields and armour come with their own materials and skills (07-10 §3.1).
- **Skill comes by practice.** Fighting is a skill gained by fighting (ADR-0006 §2). Drill
  waits until someone has a reason to ask for it.
- **Plan §5.5's Lanchester-style model is replaced** by per-minute choices and blows. The
  Lanchester laws stay as tests: a gate-limited exchange keeps the linear law, and more people
  behind the gate add no blows (13-07 §1.2, §4).

### 5. Bands and service by law

- **Two policy kinds are appended:**
  - *send a band*: its purpose, the target polity, a leader, and whether households owe
    service;
  - *against raiding*: a raid on another polity is a wrong the polity's own gathering hears.
- **Service owed** is an ADR-0015 obligation of an appended kind: days of service a year per
  eligible adult.
  - Households equip their own, and nobody is paid.
  - A band law starts it, and the band's target bounds it.
  - The band's return or the days' end discharges it.
  - Each household chooses who goes, and may keep its share back as a levy is kept back.
  - 13-03 §2.3's 0, 15, 30 and 60 days are a content menu.
- **Proposals follow ADR-0013 §5.** Each is proposed by someone holding an issue it answers, on
  their household's forecast, and decided by the polity's custom with every stance kept.
- **Without such a law, raids stay possible** (§3).

### 6. Enclosures are lines of sectors, raised by law, laid on the walking grid (amends ADR-0009 and ADR-0010)

- **An enclosure** is a closed line of sectors 10–25 m long. Each sector may have:
  - a ditch, with its spoil a bank inside, from slice BG's `EarthKind::Ditch` and `Bank` lines;
  - a palisade of ADR-0009 parts that rot at the foot by wetness;
  - a gate.
- **It lowers walking-grid cells, as decks raise them.** A sector's cells move at the least of
  the ground and a climbing factor: slow, and seen by anyone keeping watch within sight. A
  gate's cells are the ground while open and the sector while shut. Routes are kept against its
  revision.
- **A gate is open or shut by its keeper's choice.** Shutting is a choice made at dusk or when a
  band is seen. Its keeper is the watch if the law says so, otherwise the nearest household.
- **It is raised by law,** an appended policy kind *enclose*. The law names:
  - the line: around the store, the hearth, or homes within a reach;
  - how work is raised: days owed, or volunteers in equal shares as for a crossing;
  - whether the watch keeps the gates.

  Each household weighs:
  - the takings and raids it believes it suffered and expects;
  - its share of the work and posts;
  - the walk round to a gate;
  - whether its home is inside.

  Households outside the line pay without its cover. Progress is the least of labour,
  materials, tools and work front, and incomplete works give partial benefit (11-11 §1.3).
- **Upkeep is work orders,** as for a bank kept by law (slice BG). Posts rot, and a rotted or
  cut gap is a gap.
- **An enclosure is not a military installation by itself** (11-11 §1.1). Measures report works
  and raids apart, and a line never attacked may still have worked.

### 7. A siege is a band camped near a shut line, and it ends by mechanism

- **Not a mode.** A siege is a band camped near a shut line. It takes what lies outside (fields
  at harvest, homes outside the line) and stops those it sees leave.
- **Camping is presence apart from residence** (amends ADR-0018 §2). Members sleep where the band
  camps, and residence is unchanged.
- **Endurance is stores and water**, computed only as a derived measure, never saved or read:
  - Inside are the households' stores and the common store. Every walk to water goes out
    through a gate unless a well is inside.
  - The band's endurance is what it carried and takes, and its members' work at home: the
    harvest calls them back.
  - F/(Q − I) is a measure, "not a surrender timer" (11-11 §1.4).
- **Entry is local:**
  - a gate opened from inside (a revolt of those who would rather take terms);
  - a rotted or cut gap;
  - climbing at night;
  - an assault where the frontage is a gate's width.

  A breach is an opportunity, not a capture (13-08 §1.4).
- **Each of 13-08 §1.8's endings is a mechanism, never a roll:**
  - **terms:** an agreement each gathering ratifies; the besieged meet inside;
  - **capture:** a hold (§9);
  - **betrayal:** a gate opened;
  - **breakout:** households leaving by a route the band does not watch;
  - **relief:** an *assistance* clause;
  - **abandonment:** members go home one by one, never all at once on a date.

### 8. Terms after force are appended clauses, and strength is believed (amends ADR-0020 §6)

- **Clauses appended** to ADR-0020's list, each performed by people:
  - *compensation*: a gift tied to named takings;
  - *takings heard*: each gathering hears its people's takings from the other's households,
    which only a polity with a law against taking can perform;
  - *no raids*: each polity holds a raid on the other a wrong its own gathering hears;
  - *truce*: a term in which neither side may propose a band law against the other; its
    expiry opens reconsideration and causes nothing;
  - *assistance*: a band when the other is raided, by trigger and term (13-05 §1.8);
  - *tribute for peace*: ADR-0020's transfer, granting nothing but the other's *no raids*;
  - *one body*: §9's merger or surrender.
- **Strength is believed, not read.** Two claim kinds are appended (ADR-0016 §3):
  - *band seen*: where, how many, which way;
  - *raid*: who, what was taken, who was struck.

  A negotiator's estimate of the other side's strength is the people of it they and their
  companions have seen, faded by age. Nobody reads the other side's numbers, stores, laws or
  bands except through claims.
- **Threat is a forecast term, never a state or a message.** A household weighing tribute for
  peace sets the transfer against the raids it expects if it refuses, read from the raids and
  bands it heard of (13-02 §4.2). A band's leader stating a demand that travels as a claim
  waits.

### 9. A polity may hold several settlements (amends ADR-0013 §1 and ADR-0018 §1)

- **A polity keeps a list of settlements,** each with since and how: founded, annexed, merged
  or seceded. Its first is its seat.
  - Membership is residence in any of them (ADR-0013 §1's rule, widened).
  - Each settlement keeps its hearth and its own claims.
  - ADR-0018 §1's daughter founding is unchanged: a settlement founded by people of another
    polity founds its own.
- **The body gathers at the seat's hearth.** People of the far village walk there or are
  absent, so 13-04 §1.7's deadlock and domination arise from distance. The watch keeps rounds
  where its holders live. Common stores stay where they were, each kept as now.
- **A hold from outside is ADR-0017 §4's test,** applied to a band. A band holds a settlement
  when, for `hold_days`, its members keep the hearth and the store and nobody of the settlement
  stands against them. A hold changes nothing by itself.
- **What follows is a law of the holder's own gathering,** decided by its custom with every
  stance kept. An appended policy kind *annex*; leaving or seeking terms are existing moves.
- **Annexation closes the held polity's deciding.**
  - Its record is kept, and its last custom version names the polity that took it.
  - Its residents become members of the victor's polity.
  - A founding window (ADR-0017 §5) lets the victor's body weigh each of the old laws.
  - The old common store stays where it was, now the victor's, recorded on an appended ledger
    channel.
  - The victor's people know the annexed households' stores only as they knew them before
    (13-03 §1.2).
- **The same change by agreement** is the *one body* clause: a surrender or a merger, ratified
  by both gatherings, naming whose custom governs. Settlements, records and laws are kept, and
  each old polity's laws are weighed in a founding window.
- **Afterwards compliance is each person's choice:** levies paid or kept back, gatherings
  attended or not, petitions, refusals and revolts. Nothing rises toward loyalty (13-03 §5.6).
- **Secession is a revolt scoped to one settlement** of a polity of several (ADR-0017 §4).
  - The faction's program is a body of that settlement's residents.
  - It holds when the store kept there and the watch there stand with it, and more adults there
    stand with it than with the polity's body, for `hold_days`.
  - The new polity founds with a copy of the body, as a daughter does.
  - Leaving together (splinter founding) stays secession's other form.
- **Vassalage and federation are clause bundles in v0.** Vassalage is transfer, *assistance* and
  *no raids*; the *tributary* label already reads it. Federation is *assistance*. Decision-right
  clauses, client governments and a competence matrix wait.

### 10. What war leaves behind is in the systems that already keep it

- **Deaths** keep `Cause::Violence` and their encounter, which records whether each person was
  fighting. Wounds keep their days.
- **Flight.** A household raided, or one that has heard of raids near it, reviews where it lives
  at once (ADR-0018 §5's event review), with safety among the terms. The residence reason
  *fled* is appended. The yearly review weighs raids heard of too.
- **Plunder moves on an appended ledger channel.** Raiders leave seed, as `take_goods` does. A
  switch for taking seed is measured later with 13-09 §4's test: a raid before planting against
  one after harvest.
- **Grievances and views.** Those raided hold grievances against the raiders' households
  (appended `Wrong`s) and the view that the other polity *harms us*. Where a band was sent by
  law, they also blame its body (amends ADR-0020 §4). A broken *no raids* clause counts against
  *keeps its word*.
- **Captives wait.** Capture needs custody and household membership the kernel lacks (13-03
  §1.9; 06-10 §4.A).

### 11. The boundary

- **Saves** gain:
  - bands with their stages;
  - encounters with two sides and each minute's answers as harms;
  - service obligations;
  - enclosure lines, sectors, palisade parts and gates;
  - camps;
  - each polity's settlements with since and how;
  - appended enums.

  The schema is bumped. A save from before loads with each polity holding its one settlement as
  its seat, and no bands.
- **Content** gains:
  - band purposes and frontages;
  - the wooden spear and its making;
  - service days;
  - the new policy kinds and clauses;
  - palisade parts;
  - the memory window for raids.
- **The wire** shows:
  - bands walking;
  - encounters told in plain verbs;
  - lines with sector condition and gates open or shut;
  - each polity's settlements with how and when;
  - a siege page with each side's food at current use, never "siege progress".
- **Gate B** (ADR-0011 §5) gains bands, encounters and deaths by violence, with wide
  tolerances.

## Consequences

- Easier:
  - a raid is explained person by person: who was hungry or wronged, who called, who came, what
    they carried, who struck whom;
  - contested ground can finally cost something;
  - a siege's outcome traces to stores, water, gates and gatherings;
  - one polity over several settlements opens M7's federations and client governments.
- Harder:
  - every system that reads "the polity of this settlement" must read the polity's list;
  - minute stepping while people fight;
  - camps;
  - a gathering whose members live kilometres away;
  - plausibility is hard to judge, since raids may never form, or form whenever someone is
    hungry. The funnel is counted before anything is tuned (the brief's §3).
- Forbidden:
  - a raid, war or siege drawn from a calendar, a rate, prosperity or the time since the last;
  - an issue weighing toward a band;
  - a battle score, casualty breakpoint or Lanchester resolution;
  - fighters, food or weapons that come from no household;
  - a siege timer;
  - compliance that rises with time;
  - any choice reading the other side's numbers, stores or bands except through claims;
  - an annexation that reveals stores.

## Alternatives considered

- **A war state between polities with war goals** (plan §5.5). ADR-0020 §1 forbids a state that
  causes anything, and 13-05 §5.4 tracks raids and negotiations apart from it. The label is
  enough.
- **Lanchester resolution of battles** (plan §5.5). 13-07 §1.2 finds no constant-coefficient law
  fits, and keeps them only as benchmarks. They stay as tests of the per-minute model.
- **Raids only by law.** A treaty could then never be undermined by its own side. That is
  13-05 §1.2's central case, and unauthorized raids are how factions embarrass rulers.
- **Annexation as tribute and agreements only,** keeping one polity per settlement. 13-03 §5.1
  asks that annexation modify specific rights. The plan's demo names annexation, and a body
  that cannot gather its far members cannot show 13-04's deadlock.
- **A body that meets at each hearth by turns.** It is fairer to the far village, but it hides
  the cost of distance that 13-04 §1.7 says shapes unions. A custom amendment can later move
  the seat.
- **Captives in M6.** Early farming massacre evidence shows whole communities attacked (13-07
  §3). Custody needs household membership the kernel lacks, so strikes on those not fighting
  are recorded as choices, and capture waits for M7.

## Revisit when

- Polities can pay (minting, M7): hired bands and soldiers' pay.
- Courts can bind outsiders, or decision-right clauses are needed (M7).
- Masonry walls, heavy carpentry and siege machinery are needed (M8).
- Animals can be ridden or draw loads.
- Captivity has custody to rest on.
- A lived world shows sustained war that labels and bands cannot describe.
