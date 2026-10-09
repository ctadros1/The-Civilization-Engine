# Institutions, Law and Social Change

This guide describes the M4 systems in the integrated `main` tree at [`eb00281`](https://github.com/ctadros1/The-Civilization-Engine/commit/eb00281). The design is governed by [ADR-0013](../../decisions/0013-polity-offices-laws.md) through [ADR-0017](../../decisions/0017-factions-episodes-regime-change.md). The detailed implementation and run record is in [PROJECT_PLAN.md §9](../../PROJECT_PLAN.md#9-decisions-log).

## Ownership and core rule

Each settlement owns a polity and its state. Content supplies reusable policy, office, issue, norm, value and ideology vocabulary. A polity stores values over that vocabulary: who may participate, what constitutes a quorum, how the body decides, which laws are active and what offices exist. Labels such as "council community" or "oligarchy" are derived descriptions. No simulation decision reads a label or sets a regime as an outcome.

The main Rust ownership is split across:

| Concern | Implementation |
| --- | --- |
| Polity, law pipeline and constitution | `kernel/crates/civ-agents/src/polity.rs`, `population/polity.rs` |
| Sparse ties, standing and notables | `ties.rs`, `standing.rs`, `population/ties.rs` |
| Taking, witnesses, beliefs and cases | `crime.rs`, `population/crime.rs`, `population/cases.rs` |
| Watch, collection and force | `population/watch.rs`, with incident/case handling in `population/crime.rs` and `cases.rs` |
| Claims, grievances, opinion and norms | `word.rs`, `opinion.rs`, `norm.rs`, `values.rs`, `ideology.rs` and matching `population/` modules |
| Factions, episodes and programs | `faction.rs`, `population/faction.rs`, `population/force.rs` |
| Observer views | `kernel/crates/civ-sim/src/frames/government.rs`, `frames/order.rs`, `frames/standing.rs`, `frames/word.rs` |
| Content vocabulary | `kernel/crates/civ-content/src/policy.rs`, `norm.rs`, `value.rs`, `ideology.rs` |

The observer renders records and submits commands. It does not choose sponsors, votes, sanctions or sides.

## Ties, standing and institutional deliberation

People hold sparse, directed ties. Recorded interactions add evidence in domains such as provision, craft, word and counsel; evidence has its own fading behavior. A person's view of another is not assumed to be reciprocal. Standing is derived by asking what the other adults in that settlement hold of the person. The same person can therefore have different standing for different audiences.

Notables are a compute tier selected from influence and standing with hysteresis. They review institutional moves more often than the wider body; elders of affected households can also weigh in on relevant proposals. The tier is not an office, legal privilege or fixed political class. ADR-0014 records the rationale, selection and boundary.

Institutional moves are compared against doing nothing. A household forecast estimates how a proposal affects its own material position; ties affect whose counsel a sponsor regards and who is expected to support a move. A softmax samples among eligible options. This bounded decision rule keeps the kernel's choice explainable without making it an optimizer that knows the future.

## From issue to law

The M4a pipeline starts when an authored issue is present, such as an expected food shortfall. A sponsor may weigh proposing a policy template at one of its authored levels. If a proposal is made, a gathering is called. Adults who learn of it decide whether to attend; the active constitution determines who may come, the quorum and the rule for passing. Each participant's stance and reasons are recorded. Once passed, a law becomes active and travels through ordinary social contact.

The law history is append-only. It retains the proposal, sponsor and issue; terms; who heard and attended; the recorded stances; the body's decision and tally; who knows the resulting law; and what was paid, withheld, supplied or left outstanding. The Government view derives plain-language descriptions from these kernel records.

The polity's common store is a ledger holder. Levy and relief are transfers between household stores and the polity, with explicit channels. A household may refuse or be unable to pay according to its food outlook and what it believes; relief is drawn from what the store holds. The policy does not create goods. The storekeeper is an office created through a law, and its appointment, departure and succession are part of law history.

Constitution changes are proposals too. Authored options alter one part at a time: membership, quorum or decision rule. The current body judges the amendment; if passed, the new values apply and the plan records which amendment introduced each constitution version. The amendment is not an external mode switch.

## Crime, evidence and obligations

The model keeps three records distinct:

1. **Incident:** what actually happened, where and between whom.
2. **Knowledge and belief:** who saw or heard about it, what they believe and how certain they are.
3. **Case and obligation:** whether someone brought the claim to the polity, what the gathering found, and what the finding requires.

A taking is a person's scored choice from their needs, expected gain, perceived chance of being found out, and the cost of a sanction they believe will follow. Seeing a taking does not automatically create a public case: witnesses decide whether to tell or bring it. The gathering hears what is presented under its procedure and its participants decide the finding. Belief and evidence remain attributable to people rather than becoming omniscient world state.

A finding may create an obligation such as restitution or compensation. An obligation is a durable amount owed to a named party, not an instantaneous punishment. It can be paid through the ledger, remain outstanding or lapse when the party can no longer receive it. A watcher can decide to investigate or collect what is owed. Collection can lead to an encounter in which the household members decide to comply, resist or strike. Each blow and its consequences are recorded in plain language. Corporal punishment and execution are not part of this model.

The watch is itself an office under law. A watcher's rounds can expose incidents, but observers and simulation systems do not retroactively make every event known. Multiple watchers divide what remains unguarded. Curfews are laws that people may keep or break; the watcher does not automatically stop every violation. Closures of resource places, damage to property and seizure of a faction's store are not implemented at this baseline.

## Claims, grievances and political organization

A claim is a shared record of an event; hearing is a personal, sourced record. A gathering call, for example, reaches people by contact rather than giving every adult perfect awareness. A grievance is personal and durable: it names the harm, the party blamed and the expectation or law the person believes was violated. Its activation fades and can be renewed by reminders or by hearing about it. Relief can make food-related grievances good; the record keeps their origin.

Adults hold issue positions anchored to their household's circumstances. Companions can move positions through talk. Values are slower dispositions, norms describe expectations and beliefs about compliance, and ideologies are authored explanations with commitments and possible policy programs. They have different roles in proposal and deliberation; an ideology is not an imposed identity or a scripted faction.

A faction is an organization people may found or join when their grievances and ties give them reason. It has a treasury on the same ledger and may support members who need food. Its organizer may call a petition; the gathering hears it through contact and decides it as a law proposal. Members may also choose a coordinated refusal to pay a levy. Episodes are records with participants and outcomes rather than a single world-level "unrest" meter.

Revolts and coups emerge from choices and coordination. A revolt can replace the polity's current body with a program derived from the decisions people saw and the laws its supporters would prefer. A coup is limited to offices that hold force; in the implemented design, a watch with several keepers can contest the body's authority. If successful, a new constitution version is recorded and the new body bargains over the inherited laws. The system does not schedule a revolution or target a preferred regime count. See ADR-0017 for the boundaries and deferred cases.

## Observer interventions and derived views

The observer can introduce a true claim to one person's hearing, tell someone of an ideology, send an adult agitator into a settlement as a newcomer, and bless or curse a person for a time. Blessings and curses affect a bounded set of keyed chances (such as illness or discovery); they do not rewrite a decision, tie, vote, detection event or punishment. Each intervention is recorded, and inspector descriptions summarize effects from what people did.

Government, Takings, Standing and inspector panels are projections of kernel records. They expose histories, reasons and current state but have no independent authority. The map and panels can be empty or lack a view where the record contains no event; absence is not filled in by a narrator.

## Limits at this baseline

- This is a small authored set of policies, issues, offices and social values, not a model of modern states, elections, bureaucracies or general-purpose legal codes.
- A passing law, a revolt, an ideology becoming common or a regime label is contingent on the initial world and people's choices. The demos document what their selected runs did, not a guaranteed outcome.
- The M4 fifty-year dashboard's latest recorded run has one failed food-price row in one world. The detailed result, other rows and uncertainty are preserved in the plan's decision log.
- Some designed cases remain deferred, including closures, property damage and seizure/retention of a faction's store.

## Further reading

- [ADR-0013: Polity, offices and laws](../../decisions/0013-polity-offices-laws.md)
- [ADR-0014: Ties, standing and notables](../../decisions/0014-ties-standing-notables.md)
- [ADR-0015: Incidents, cases and obligations](../../decisions/0015-incidents-cases-obligations.md)
- [ADR-0016: Grievances, claims, opinions and interventions](../../decisions/0016-grievances-claims-opinions.md)
- [ADR-0017: Factions, episodes and regime change](../../decisions/0017-factions-episodes-regime-change.md)
- [M4 implementation and run journal](development-history.md)
