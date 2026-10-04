# ADR-0008: Knowledge carried by people

Status: Accepted
Date: 2026-10-04
Milestone: M3b
Amended: 2026-10-04, slice M, to match what was built: §1's bootstrap test, §2's derived
settlement knowledge, §4's upbringing at the first work, §5's loss check

## Context

Until M3b everybody can do everything the content describes. A founder, a child who has just
reached the age for the work and a newcomer all make axes, build huts and grind grain alike,
and skill (ADR-0006 §2) sets only how fast they work and how long their tools last. M3b needs a
technology graph v0 of about 30 nodes, discovered by named people, spreading between them and
lost with them, and the god tool *introduce a technology* (plan §5.6, §7). Saves will keep what
each person knows for the life of a world. Panels and the C interface will show it.

Reports read: 07-01 (invention), 07-02 (diffusion and loss), 07-03 (the graph), 07-11
(knowledge institutions), 06-08 (learning and apprenticeship), 04-05 (social learning), 08-17
(guilds and apprenticeship), with 07-04, 07-05 and 07-07 for the nodes. Their main points:

- **Knowledge lives in people.** A settlement's capabilities come from its people,
  facilities and records. Never set a flag such as `civilization.knows_iron` or take the union
  of a group's knowledge (07-02 §1.1; 07-03 §3.1, §3.3; 07-11 §5.2).
- **A node is a practical capability with executable content.** It is phrased "a competent
  actor can …". "Unlocks another technology" or "+5 % output" is not enough (07-03 §3.2,
  §4.1).
- **Requirements are alternative complete routes,** an OR of ANDs (07-03 §3.3). Discovery,
  learning, operating and adoption conditions are separate: losing a fuel supply stops
  production, not understanding (07-03 §3.4).
- **States matter:** awareness, competence and use differ (07-02 §1.1; 07-03 §5). Seeing,
  watching and supervised instruction teach different amounts (07-02 §1.2).
- **Discovery comes from bounded practical activity:**
  - The chance of resolving a problem is P = 1 − exp(−(ln 2 / E50)·G·E_eff). E_eff is
    qualified experiment hours and G a feasibility gate (07-01 §5.3; 07-11 §5.2).
  - Priors only: E50 tested at 10³, 10⁴ and 10⁵ hours. Routine experimentation is 1 % of
    working time, tested 0–5 % (07-01 §2.3); ordinary craft workers have 0–10 % protected time
    (07-11 §2.2).
  - No universal rate of discovery is defensible (07-01 §2.2). Population counts only through
    the people doing the work (07-01 §1.4).
  - Need steers which problems get attention (07-01 §1.6). An impossible configuration is
    never rerolled until it works (07-01 §5.2).
- **Learning takes practice beside someone who knows:**
  - 20–200 learner-hours for a narrow observable procedure, 500–5,000 for a specialised stage
    (07-02 §2.3).
  - One to three learners per teacher (07-02 §2.3; 07-11 §2.2).
  - Teaching takes 5–25 % of a teacher's time unless combined with production (08-17 §2.2).
  - Early farmers learn in households and kin, not guilds (08-17 §3.1; 06-08 §1.1); founders
    arrive knowing their farming (06-08 §3).
  - A teacher's death stops learning but keeps what was learnt (08-17 §1.3).
- **Loss is a broken reproduction process, not a population threshold.** There are four
  states: production suspended, expertise endangered, no local practitioner, unrecoverable
  (07-02 §1.7, §5.4). There is no automatic forgetting: start at zero (07-02 §2.3). Techniques
  that depend on a lost one are not deleted (07-03 §5).
- **Scale through sparse, event-driven records:** no scan of every person against every node
  (07-01 §5.5; 07-02 §5.5; 07-03 §6).

## Decision

### 1. Techniques are content

- A **technique** is a content kind (`kind = "technique"`, ids `<pack>:technique/<name>`).
  It gives:
  - a name, and what a competent person can do, in a sentence;
  - the skill whose practice it is (`domain`);
  - its prerequisites, as alternative routes, each a set of techniques (empty for none);
  - what discovers it: the activities whose practice counts toward it, the goods a household
    must hold to try it (the gate G), and E50 in qualified hours;
  - the hours of work beside someone who knows it that teach it (`learn_h`);
  - whether children brought up in a household that knows it learn it on reaching the work's
    age (`upbringing`).
- Recipes, activities and building programs (and, with grammar v2, building parts) may name one
  technique each. Only a person who knows it may do that work, or someone working beside a
  person who does (§4). Work that names none is open to everyone, as all work was before.
- The content validator refuses:
  - an unknown technique;
  - a technique that gates nothing;
  - cycles in prerequisites;
  - a recipe that can never be worked because an input or tool comes only from recipes that need
    it. This is 07-03 §6's bootstrap test, applied to every recipe as a fixpoint over the goods
    the world renews (gathered, harvested, or made by recipes that can themselves be worked).

  The content API becomes 9.
- The people profile's `[knowledge]` table gives the share of grown founders who know each
  technique. A band with a share above zero always brings at least one knower. Families sent by
  the observer draw the same way.
- Skills stay one per domain (ADR-0006 §2): knowing is a gate, skill is how well. Per-technique
  proficiency (07-02 §5.1) is left for later.

### 2. What each person knows

- Each person keeps a short list, sorted by technique. Each entry holds:
  - the hours learnt so far (it is known at `learn_h`);
  - since when;
  - how it came: founder, upbringing, taught by whom, found, or introduced by the observer;
  - when it was last practised.
- An entry with no hours is awareness: the person has heard of it or seen it. One with fewer
  than `learn_h` is learning.
- Nothing is forgotten: unpractised knowledge and skills do not fade (07-02 §2.3).
- What a settlement knows is always derived from its people, never stored. Its residents are
  looked over when something asks: a death or a departure, or the knowledge query. In villages
  of hundreds that is cheap. An index from technique to knowers waits until it is not.
- The settlement keeps a short history for each technique that was ever known there: who first
  knew it there, when and how, and when it was lost and who knew it last. That history cannot be
  derived once its people are gone.

### 3. Discovery

- Discovery is a draw at the end of a work session. It applies when the session's activity
  counts toward a technique the person does not know, one of its routes is wholly known to them,
  and the gate's goods are at home.
  - The session's qualified hours are its hours times the share given to experiment: 1 % in
    routine work (07-01 §2.3), and all of the hours of a session spent trying (below).
  - The chance is 1 − exp(−ln 2 · qualified hours / E50). Because the hazard is per hour,
    splitting a session changes nothing (as with ADR-0003's hazards).
  - Draws are keyed by person, technique and session start, so runs are reproducible and
    independent of thread scheduling.
- Practice alone finds little: crafts take under an hour a day of a village's time. A new
  behaviour, **try**, spends spare hours experimenting toward a technique. Its worth comes from a
  household problem the technique answers, in measures the engine already keeps: goods lost to
  spoilage, hours spent on a task, work waiting on a good. A problem nobody has gets no
  attention.
- The finder knows the technique at once, at their own skill. 07-01 §1.2's gap between a first
  success and a reliable practice is simplified away.
- The chronicle records the find. The world records the first find anywhere, and each
  settlement its first knower, so first-in-world, first-here and independent finds stay
  distinct (07-01 §1.3).
- A technique once known and then lost is found again the same way. Someone aware of it
  (§2) counts the hours of trying at a higher rate, which is 07-02 §1.7's reconstruction route.
  The rate is a tuning value.

### 4. Teaching

- **Upbringing.** A child learns each `upbringing` technique on reaching the youngest age of
  the work it gates, when someone in their household knows it. This stands in for community
  learning of everyday work (06-08 §1.1).
  - It happens at their first such work or at the start of the next day, whichever comes first,
    so a child is never held back from work their household would have them do.
  - Work first done later than a year before the age of keeping a household is learnt at home
    in that last year (a tuning value), so nobody leaves home without their household's work.
    Making an axe, from 16 where people may keep a household from 15, is one example.
- **Working alongside.**
  - A person who does not know a technique may do its work only beside someone who does:
    - a member of their own household at the same place and work at the same time;
    - or, for hired work, at a workshop whose owners know it (supervised work).
  - Each hour of that work counts toward `learn_h` and practises the skill as usual.
  - A teacher takes at most two learners at once (07-02 §2.3). Teaching does not slow the
    teacher in v0; the cap stands in for its cost (08-17 §2.2's 5–25 % is a later
    refinement).
  - Learners are drawn to such work by a learning term in their decision.
  - Learning is kept when the teacher dies or the learner leaves (08-17 §1.3).
- The chronicle records learning only for techniques that are not `upbringing`, so the spread
  of a craft can be followed without everyday work flooding it.

### 5. Loss

- When a person dies or leaves a settlement, each technique they knew is checked against the
  settlement's remaining residents. If none of them knows it, it is lost there.
- The chronicle records the loss and names the last knower. It says whether anyone there is
  still aware of it or learning it, and whether goods or buildings made with it remain
  (07-02 §5.4's flags).
- Nothing else changes: people keep techniques whose prerequisites were lost (07-03 §5), and
  goods and buildings made with it remain.

### 6. The god tool

- `IntroduceTechnique { person, technique, aware_only }` makes a living person know a
  technique, or only hear of it. The source is recorded as the observer, and the chronicle
  records it (plan §2).
- The person needs no prerequisites, because learning skips the discovery chain (07-03 §3.3).
  They keep their current skill. What the work needs to operate (goods, tools, deposits) still
  gates its use.

### 7. Saves and boundary

- Saves keep each person's entries by technique id (a dictionary in the `People` section, as
  skills are) and each settlement's technique history. The schema becomes 13.
- Saves from schema 12 and earlier load with every person given the founders' knowledge for
  their age, drawn as a founder's is, as skills were at schema 9.
- The wire gains:
  - the techniques in `Welcome`;
  - a person's entries in `PersonInfo`;
  - a knowledge query per settlement: each technique's knowers and learners, its history and
    its state in words;
  - a knowledge revision in the snapshot;
  - the `IntroduceTechnique` command.
- New chronicle kinds (found, learnt, lost and introduced) and the `try` behaviour are
  appended. Nothing is renumbered.

## Consequences

- Children learn their household's work, crafts can concentrate in a few households and be sold,
  and a craft can die with its last practitioner.
- A world's repertoire can differ from another's: by the founders' draws, by what was found and
  by what was lost. Its era is never a setting.
- Household costs count a recipe only at the skill of members who know it. A household that
  knows no way to make a good buys it, learns it or does without.
- Every decision that offers work checks a gate. The check is a lookup in a short sorted list.
- Forbidden:
  - knowledge held by a settlement, polity or culture rather than by people;
  - research points, or discovery driven by population size or the calendar;
  - deleting techniques downstream of a lost one;
  - forgetting by a timer.

## Alternatives considered

- **Knowledge per settlement (a set of known techniques).** It is cheaper, but it cannot lose a
  craft with a person, teach it, or tell who found it. The research rejects it explicitly
  (07-02 §1.1).
- **Per-technique proficiency in place of per-domain skills.** It is closer to 07-02 §5.1, but
  it doubles what each person keeps and changes ADR-0006's calibrated skills. Knowing as a gate,
  with domain skill as how well, keeps M3a's numbers.
- **A yearly discovery roll per settlement** (plan §5.6's first sketch, chance per year from
  population and specialists). It ignores who was working at what, and it is a population
  multiplier, which 07-01 §1.4 and §5.3 warn against. Session hazards attach finds to named
  people at no extra cost.
- **Teaching as a separate activity.** It would need two people's schedules joined. Working
  alongside reuses the work people already choose and charges the learner's time.

## Revisit when

- Writing or other records arrive: knowledge carriers that are not people (07-02 §1.8).
- Several settlements trade or migrate (M5): diffusion through contact, and loss that is local
  rather than global.
- Guilds, schools or patrons (M4 and later): access, secrecy and protected time as
  institutional rules.
- The Accelerated mode (M3c) needs a daily stand-in for working beside someone.
