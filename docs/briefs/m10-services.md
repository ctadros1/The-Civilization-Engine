# M10 design brief: capability rungs, healthcare and education as services

**Scope.** Plan §7's M10 ("Industrial content") lists "steam, factories, rail, gas light; capability
rungs for industrial services; healthcare and education as full services; fiat money". Plan §1 keeps
healthcare and education "a single abstract stat (healer availability, literacy/skill rate) until its
later milestone", and §8's open question 8 names M10. Sibling briefs own steam and factories, rail
and gas, and money. This one covers what the plan means by capability rungs and how a service climbs
them; care beyond the household, birth care, inoculation, isolation, a health office and sickhouses,
and what each changes in sickness and death; and education: upbringing and working alongside as
today, then teaching, letters and reckoning, schooling provided by a household, an organization or a
polity, and who may hold which work.

Reports read in full: **05-05** (healing and health institutions), **06-08** (education,
apprenticeship and literacy), **07-11** (institutions of knowledge), **05-04** (the urban penalty and
sanitation); skimmed: **05-03** (epidemiology), **12-02** (sanitation). ADRs checked: 0006, 0008,
0013, 0021, 0022, 0026, 0032.

**What exists to build on** (checked in the code; paths under `kernel/crates/civ-agents/src/`):

- **Episodes are truth.** `Episode`, `Course`, `Care` (`sickness.rs`): each infection keeps its
  acquisition record, shedding and illness days, severity and care given. `Care::rr` takes the least
  relative risk a carer brought that day (supportive `care_rr` or a treatment's), in proportion to the
  share of `care_h_per_day` given. `sickness_day` (`population/sickness.rs`) draws each severe day's
  death at `severe_death_per_day × care.rr`, then sums each person's household and water hazards and
  draws once. `DiseaseRoute` has `Water` and `Household` only (`params.rs`); cholera and bacillary
  dysentery are authored (`content/core/disease/`).
- **Tending is the household's only.** `tend_facts` counts care owed to ill members of the carer's
  own household; `tended` shares the hours among them and records the least rr of any treatment the
  carer knows (`population/sickness.rs`), called at the step's end (`population.rs`,
  `Behavior::Tend`). ADR-0021 §5's carers from "kin or those who regard it" were not built. Cholera's
  `[[treatment]]` is `fluid_replacement` at 0.05 against supportive care's 0.95
  (`content/core/technique/fluid_replacement.toml`: E50 100,000 h, learnt in 20 h of tending).
- **Childbirth is one flat draw:** `maternal_death_per_birth` = 0.0075 a live birth
  (`population/life.rs`; `content/core/people/early_farmers.toml`, from 05-01 §2.2's England
  1750–1800), `Cause::Childbirth` (`history.rs`). Nobody attends a birth.
- **People act on what they see.** Sickness seen makes claims, suspicion of a source is each person's
  tally (`suspicion.rs`; ADR-0021 §6). `CloseSource` and `DigWell` exist; quarantine was deferred
  ("Water only, v0") and is not among `PolicyKind`'s 32 kinds (`polity.rs`).
- **Knowledge is held by people** (ADR-0008). `Know` (`person.rs`): known or hours toward `learn_h`,
  since when and how (`Founder`, `Upbringing`, `Taught`, `Found`, `Observer`, `Seen`). `gate`
  (`population/knowledge.rs`) opens work to a knower, to a learner beside a household member at that
  same work now (`max_learners` = 2), or to a hired hand at a workshop whose owners know it;
  `bring_up` teaches `upbringing` techniques at home daily. Skills grow per domain by 06-08 §5.2's
  rule (`SkillDef::practised`, `params.rs`). A law may already need its sponsor to know a technique
  (`RuleDef.needs`, `polity.rs`).
- **No literacy, writing, record, healer or school exists** anywhere in the kernel, and no dashboard
  row measures one (`civ-host/src/dashboard.rs`). Plan §1's abstract stats were never built: M10
  converts nothing.
- **Offices are laws with a holder** (ADR-0013 §2; `law.holder`): storekeeper, watch, mint, hearer;
  their work scores as duty (`MakeWorth::Office`, `decide.rs`). The store can pay coin an hour for
  common works (`content/core/policy/pay_work.toml`) and raise a building of an authored program
  answering a loss the polity measured (`raise_building`, ADR-0031 §6; `yourselves_roof_a_hall.toml`).
- **Firms sell goods, not services.** The household workshop is the only form (ADR-0006 §5); hired
  hands earn wages; none of the ledger's 26 channels (`ledger.rs`) is a fee for work done for a
  client. Organizations by stake collect dues and give mutual relief (ADR-0026 §2).
- **Household-resolved days** (ADR-0032 §2): `Tend` is shared (`population/resolution.rs`,
  `SHARED`). Members are parked at home with no target, so `working` (`population/knowledge.rs`) is
  never true while planning: **as I read it, learning a non-upbringing technique beside a household
  member is closed in a household-resolved settlement**, while upbringing, hired learning and a
  treatment learnt while tending stay open. Read, not run.

## Where the reports agree

1. **Access to particular services, not a mortality bonus.** 05-05 (opening; §2.2: not "healers
   reduce all mortality by 5%"): effects are condition-specific and apply to care actually delivered,
   p = p⁰(1 − a·e) (§5.1). "Different services affect different outcomes" (05-03 §1.4).
2. **Capabilities, not a ladder.** 05-05 §4 shows levels but stores "a capability vector internally:
   nursing | birth care | procedures | medicines | prevention | diagnosis" and says "Do not require
   every preceding row". Provider types "should not be successive levels of one universal upgrade
   chain" (06-08 §5.5); "Do not require the sequence: household teaching → guild → university →
   academy" (07-11 §5.5).
3. **A building supplies none of it.** A provider needs an instructor, a curriculum, teaching time and
   resources (06-08 §1.4); usable beds are min(physical, staffed, supplied) (05-05 §1.6).
4. **Hours are real.** Care consumes someone's labour (05-05 §1.1); capacity is provider-hours over
   required hours (§5.3); learning consumes "time and scarce teaching capacity" (06-08, core
   recommendation), and a mentored hour counts once (§5.2).
5. **Knowing, understanding and doing are separate** (07-11 §1.1: tacit and codified; 05-05 §1.2;
   06-08 §1.2: skill, completion and permission to practise). A credential is "a social or legal claim
   about competence" (06-08 §5.1).
6. **Literacy matters where text is used:** "for a nontextual manual task, the initial direct
   literacy bonus can be zero" (06-08 §5.6); basic and advanced knowledge differ (§1.6); literacy can
   be delegated to a scribe (§1.5).
7. **Institutions come from people's reasons** (07-11 §5.5); a government acts on "smell, blocked
   streets, disputes, or visible illness" (12-02 §1.7); recognizing, authorizing, financing and
   operating are separate achievements (05-04 §3.3).
8. **Care can harm,** through existing routes, not "an unexplained universal hospital penalty"
   (05-05 §1.6; §2.1: Semmelweis's 11.3 % against 2.0 %).
9. **No double count:** life tables already include prevailing care, p(a) = p_ref·(1 − a·e)/(1 −
   a_ref·e) (05-05 §5.2); 05-04 §5.3.

**Where they differ, or differ from the plan:**

- **The plan's ladder.** §5.4 defines services "with a capability ladder unlocked by technology",
  each rung setting "response speed, suppression rate and coverage"; §5.6 lists "service rungs" in a
  node's footprint; §8's scope risk 1 counts them in the content multiplier. ADR-0021 and ADR-0022
  already rejected ladders for water and fire ("Alternatives considered"). §1 keeps the word and
  changes its meaning.
- **Literacy and discovery.** Plan §5.4 has education "also feed technology discovery", and §5.6
  drives discovery partly by literacy. 06-08 §5.7 finds no "inventions per literate person" rate and
  ADR-0008 forbids population-driven discovery; discovery stays ADR-0008 §3's session hazard.
- **Class size:** 15–40 learners an instructor (06-08 §5.3) against 5–30 a session (07-11 §2.2), both
  design priors.
- **The baseline.** 05-05 §5.2 adjusts a life table by reference coverage; ADR-0021 §5 keeps disease
  deaths as excess over Siler. Birth care meets the first rule, since 0.0075 already includes care.

## 1. Capability rungs, read for M10

**A rung is an authored capability with its footprint**, not a level: a technique, good, building
program, office or policy (plan §5.6, "node only with its footprint") that sets one property of one
function: who can be reached (access, a), what it does to which condition (effectiveness, e), what
it costs in hours and goods, and where. A settlement's standing on a service is derived, never
stored: 05-05 §4's vector counted from its living people, buildings and laws, put in words and
labelled after the fact as regimes are (ADR-0013 §6). Nothing reads the label.

**An industrial service climbs** as fluid replacement arrives today: someone finds the technique, is
taught it or is given it (ADR-0008 §3, §6); a household or firm makes its goods; a provider offers
it; clients come or do not. Industrial rungs differ only in footprint: made goods (vaccine matter,
printed sheets), dedicated buildings, paid hours, offices.

| Function | Early rungs | Industrial rungs (each its own footprint) |
|---|---|---|
| Care of the ill | household tending; supportive care | care sold by the visit; a sickhouse with paid nursing |
| Birth | unattended, or a kinswoman | birth attending; clean hands |
| Prevention | avoiding a suspected source | variolation; vaccination with made matter; isolation by law; a register |
| Instruction | upbringing; working alongside | teaching sessions; letters and reckoning; printed records |

No rung requires another unless its footprint does: vaccination needs matter, not variolation; a
sickhouse needs nursing hours, not a physician. Where gas light (its sibling brief) lengthens a
room's usable hours, classes and nursing gain evening hours through `daylight_only` on their
activities, with no rule of their own.

## 2. Healthcare

### 2.1 Care beyond the household

- **Calling.** A household with someone ill and its own carers short of the care owed (`tend_facts`'s
  measure), or with a birth due, may send a member to ask a carer it knows: kin, a tie, someone seen
  tending or named for it in claims (ADR-0016 §3). `Call` (appended) is shaped as `Ask`: a walk to the
  carer's door leaving a request with a fee. The fee is the household's own forecast of what the care
  is worth (days it believes the sickness costs, `w_death`; ADR-0021 §7), capped by what it holds.
- **Tending another home.** `Tend` gains a target, a household that called; `tend_facts` reads
  requests indexed by carer household. Care, the treatment's rr and the carer's exposure
  (`Acquired::Household` of that home) follow as today.
- **Paying.** A fee moves through an appended `Fee` channel after the session; kin and ties
  (ADR-0014) may ask none. A healer is someone whose household earns fees for care; "physician" is a
  label for one who knows several treatments and lives on fees. Neither is a state.
- **Reputation is what clients saw:** a carer is weighed by the walk, the fee and a tally of their
  tended who lived or died, as the household and its ties know it, never by the rr. A prestigious
  carer of a useless remedy can be sought (05-05 §1.2).
- **Household day.** `Call` is a once-a-day block at the carer's door, as `Ask` is (`ONCE`); a
  household-resolved carer sees the request at its next planning, so care starts the next day (a
  declared approximation). `Tend` elsewhere is a block whose minutes are shared among that home's ill
  when applied.

### 2.2 Birth care

- **The draw is split** into bleeding, infection and obstructed labour, each a share of the maternal
  rate (design priors), as 05-05 §1.5 asks. Infant survival is untouched: the Swedish midwives'
  maternal benefit "did not translate into detected reductions in stillbirth or infant mortality".
- **Reference coverage.** 0.0075 is p_ref at an authored a_ref: an unattended birth uses
  p_ref/(1 − a_ref·e), an attended one that times (1 − e), per complication (05-05 §5.2).
- **Techniques:** `birth_attending` (bleeding and obstruction; e bounded by 05-05 §2.1's 20–40 % for
  doubled Swedish midwives, a population figure taken as a ceiling) and `clean_hands` (infection;
  §2.1's 11.3 % → 2.0 %). An attendant who tended a fevered case within days without `clean_hands`
  raises the infection share: harm through the attendant's record (05-05 §1.6).
- **Household day:** births are drawn at midnight (`population/life.rs`), so attendance means an
  attendant who agreed by `Call`, lives within reach and is neither ill nor away that day.

### 2.3 Prevention

- **A third route.** Inoculation needs a disease it prevents. `DiseaseRoute` gains `Company`:
  shedders at the hearth expose their sampled company (`keep_company`, `population/ties.rs`), summed
  with the day's other hazards. Smallpox is authored on it: 05-03 §2.1's 10–14 days' incubation,
  §2.2's R₀ of 3.5–6 as a check, §3.2's 30 %. A household-resolved day draws one company session a
  member-day (ADR-0032 §2), so the route works in both resolutions.
- **Variolation:** an appended `Acquired::Inoculated { by }` episode of a mild course that sheds and
  can infect, dying at Jurin's 2.1 % against 16.6 % (05-05 §2.1).
- **Vaccination:** a technique and a good (`vaccine_matter`, made by a recipe); about 95 % protection,
  strongest 3–5 years then waning (05-05 §2.1), a protection record `Sickness::protected` reads. No
  matter in the stores, no vaccination: supply and uptake stay apart.
- **Who comes forward** is each household's weighing of the sickness it believes near (claims, deaths
  known), the fee, and for variolation its own risk.
- **Household day:** an inoculation is a `Tend` session at the client's home, a block for each side,
  its effect applied with the provider's block.

### 2.4 Isolation, the health office and the sickhouse

- **`keep_apart`** (policy): a household with someone ill of a disease believed to pass by contact
  keeps to its home and others keep from it until days after the last is well; kept by choice, as a
  curfew is. Passed against a waterborne outbreak it costs work and does little (05-05 §1.7): a
  legible mistake.
- **A health office** (an office law, as the watch is): its holder keeps a **register** of every death
  and sickness claim they hear and every home their rounds enter, and walks rounds as the watch does,
  seeing the abed (ADR-0021 §6). Their suspicion tallies run over the register, a wider record than a
  household's, so recognition can come earlier, or be wrong (05-04 §5.4; 05-05 §5.5: isolation
  "performs poorly when recognition occurs late"). Holding it needs `writing` (§3.3). Its pay is the
  store's, `pay_work`'s mechanism extended to an office's hours.
- **`clear_middens`** (policy): households carry midden waste beyond a set distance from any source,
  kept by choice; breaches seen on rounds become cases only if someone brings them (ADR-0025). Drains
  and collection are M11's waste.
- **A sickhouse** (program `use = "sickhouse"`, raised by law, or built by a firm or organization):
  admitted patients sleep there, so household-route exposure moves to its inmates and staff (05-05
  §1.6); care comes from staff hours; beds are min(built, staffed, supplied). The issue that opens it
  is measured as the hall's is: days a severe case had nobody to tend it (`untended`).
- **Household day:** rounds are a once-a-day block, as the watch's; admission is decided at midnight
  in `sickness_day`; staff hours are `Tend` blocks at the sickhouse.

### 2.5 What changes in sickness and death

Changed: a severe day's hazard where care or a treatment was given (existing); maternal deaths by
complication where a birth was attended; incidence where someone was protected, kept apart or
admitted; exposure for carers, attendants and inmates. Unchanged: the Siler baseline, infant
survival, hunger, and every cause no service touches. Since imports from off the map do not exist
(ADR-0021 §8), only birth care shows in a world the observer has not brought a disease to.

## 3. Education

### 3.1 What stays

Upbringing and working alongside stay as ADR-0008 §4 has them, reproducing "complex practical
skills" without classrooms (06-08 §4). Nothing below is required for any existing work.

### 3.2 Teaching, settled at midnight

- **`Teach` and `Study`** (appended): a knower holds a session of a technique at a place (a home, a
  workshop, a hall, a schoolroom) for those enrolled; a learner attends. Enrolment is a record
  (teacher, learner, technique, fee, payer).
- **Credit at midnight.** Sessions and attendances are recorded with place and minutes; at midnight a
  learner's hours toward `learn_h` are min(attended, held), shared out beyond the technique's group
  size (06-08 §5.3: 15–40 for elementary group work, 1–3 for crafts). No joined schedules (ADR-0008's
  objection to teaching as an activity), the same in both resolutions, and the daily stand-in its
  "Revisit when" asked for. Working alongside recorded the same way reopens learning beside a
  household member when household-resolved.
- **Teaching costs the teacher** their hours, ending ADR-0008 §4's "does not slow the teacher in v0"
  where a session is held.
- **Who pays:** the household by `Fee`; an organization from dues (ADR-0026 §2's service extended to
  members' children); a patron household; or the polity through an office (`keep_a_school`, a holder
  paid from the store, places open to members' children). `schooling_required` is a law kept by each
  household's choice.
- **Why a child is sent** (06-08 §1.3, no perfect forecasts): the fee and the child's forgone hours
  (`capacity_by_age`) against what letters visibly bring to people the household knows (the pay of
  offices and posts that need them) and its values. Knowing nobody whose letters paid, it sends nobody.
- **Household day:** `Teach` and `Study` are blocks at the session's place; the record is their effect.

### 3.3 Letters and reckoning

- **Techniques per script:** `reading`, `writing`, `reckoning`, separate as 06-08 §1.5 asks, never
  `upbringing`, `learn_h` from §5.3's 1–3 effective instruction years for elementary reading (a design
  prior). Founders bring none: no writing "should set textual literacy to zero" (06-08 §3).
- **What they gate, and nothing else:** offices whose work is records (the register, a clerk, a roll),
  as a rule names a technique its sponsor needs; a workshop's posts, where it requires hires to know a
  technique; and **records**, saved carriers of a technique (holder, place, maker), ADR-0008's awaited
  "knowledge carriers that are not people". Reading one makes the reader aware (appended
  `KnowSource::Read`), so their tries count at the aware rate (ADR-0008 §3); a technique marked
  codifiable may be learnt from it at a content multiple of `learn_h`. Records burn with their
  building (ADR-0022), so copies elsewhere matter (07-11 §5.4).
- **No bonus to manual work** (06-08 §5.6). A household may pay a writer by `Fee` (delegated literacy).
  **Printing** is this function's industrial rung: a press recipe copies records from paper goods.

### 3.4 Who may hold which work

What a person knows (ADR-0008's gates), what an office or post requires (§3.3), and **licences**: a law
that only knowers of a technique may sell a named service (care, attending births, inoculating,
teaching), shaped as ADR-0026 §3's privilege and kept by choice. Who knows what follows from who could
pay and who was let in; restrictions are institutional, "not ... inherited differences in learning
ability" (06-08 §1.3), so inequality can emerge and is never set.

## 4. Performance and the household day

Settlements of 2,000 live at about 0.7 ms a person-day (ADR-0032's amendment of slice DN), and every
new behaviour must name its block (ADR-0032 Consequences). Each does: `Call` and rounds once a day,
`Tend` at another home, `Teach` and `Study` settled at midnight, admissions and births at midnight.
Costs grow with participants, not people: requests by carer household, enrolments by person, sessions
and inmates by place, the register by entries. Every new fact sits behind a quiet check, as
`tend_facts` returns when `sickness.quiet()`, so a world with nobody ill, due, enrolled or teaching
scores no new option. Measure with `bench_village` at ten settlements of 2,000 before and after each
slice; Gate C (ADR-0032 §5) gains care hours, births attended and hours taught.

## 5. Guards

- No mortality, cure or productivity bonus for a healer, building, office or literacy rate; an effect
  applies to the episode, birth or learner served, by what the server knows (ADR-0021 Consequences).
- No ladder and no trigger: a rung requires only its footprint; no date, size or era opens anything.
- No choice reads an episode, acquisition record, treatment rr or another's knowledge hours.
- No credential stands for competence; no school produces discoveries (07-11 §1.3).
- Licences, compulsory schooling and isolation are kept by choice; breaches reach ADR-0025's forums.
- Labels ("physician", "school", "public health") are words for the observer.
- A world that uses none of this lives as before, digest for digest; save → load → save stays exact.

## 6. Suggested slices

1. **Care beyond the household:** `Call`, `Tend` at another home, the `Fee` channel, reputation from
   claims; a household all abed is tended by kin or a paid carer; both resolutions.
2. **Birth care:** the split maternal draw with reference coverage, `birth_attending`, `clean_hands`
   and the attendant's harm; with no attendant, the maternal rate stays within noise of today's.
3. **Teaching settled at midnight:** `Teach`, `Study`, enrolment, the midnight credit, working
   alongside recorded the same way (learning works household-resolved), group sizes, fees, dues and
   `keep_a_school`.
4. **Letters at work:** `reading`, `writing`, `reckoning`; records (awareness, codified learning,
   burning); office and post requirements; licences.
5. **Inoculation:** the `Company` route, smallpox, variolation, vaccination with made matter,
   protection records; uptake from belief.
6. **The health office and the sickhouse:** register and rounds, `keep_apart`, `clear_middens`,
   `untended`, the sickhouse's pool and capacity; the observer's service views (care given, births
   attended, enrolment and letters by age, the register) and the M10 services demo.

## 7. Open questions for the designer

1. A new `Fee` channel, or `Wage` for hours worked at the client's?
2. Letters as techniques (a gate, as proposed) or as a skill domain with a level?
3. One script a world in v0, or scripts as content from the start (06-08 §1.5)?
4. Should birth care rescale the maternal rate by reference coverage now, and what is a_ref for 0.0075?
5. Smallpox and `Company` in M10, though no world meets a disease without the observer until imports
   from off the map exist? Or imports first?
6. May kin tend unasked (they see the sickness), or only on a call?
7. Does a sickhouse need an office, or may any firm or organization run one?
8. One health officer, as proposed, or a seated board (ADR-0024 §1)?
9. Records as saved entities, as proposed, or goods once goods carry identity?
10. Does a teacher's skill change how fast learners learn (07-11 §2.1), or only whether they can teach?

## 8. What an ADR must decide

What a capability rung is (a footprint setting one property of one function), and that the plan's
ladders are read that way; services paid by fee and its channel; care, birth attendance and
inoculation as sessions at a client's home, recorded on the episode or birth; the split maternal draw
and its reference coverage; the `Company` route and protection records; the sickhouse as a contact
pool with capacity by its scarcest input; teaching settled at midnight, amending ADR-0008 §4; letters
as techniques and records as carriers, answering ADR-0008's "Revisit when"; technique requirements on
offices and posts, and licences as privileges; the appended behaviours, policies, issues, routes and
sources, each with its household-day block; and the save sections each adds.
