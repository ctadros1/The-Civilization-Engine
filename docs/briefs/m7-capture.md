# M7 design brief: corruption, capture, guilds and trade policy

**Scope.** Plan §7's M7 ("Politics deepens") lists "Full corruption and patronage" and "Lobbying and capture by firms and guilds". Its demo begins "a republic's assembly captured by a merchant guild passes protectionist tariffs". Plan §5.2 names the parts: "nepotism, institutional capture, drag on growth and discovery", "Market regulation: price caps, guild monopolies, licensing" and "Tariffs and embargoes". Plan §5.3 adds "Founding a firm: when an opportunity is perceived and there is capital and legal permission" and "Trading abroad, and smuggling around tariffs". This brief covers, for villages of 100–3,000 people with a few workshops each and trade between two or three of them:

- corruption beyond the watch's bribe: diversion, favour in naming and in relief, and how it reinforces itself or not;
- patronage and clientelism, and what they do to a gathering's votes;
- guilds as organizations: forming, membership, monopoly and licensing;
- tariffs and embargoes as laws, who gains and who loses, and evasion;
- lobbying, and how an interest comes to carry a body's votes with nothing scripted;
- drag on growth and discovery, as an outcome of those mechanisms.

Sibling briefs own elections, parties and the republic itself, courts and precedent, and credit and coinage. This one needs from them a body of elected members, a forum that hears a breach, and later a money to pay duties in.

**What exists to build on** (checked in the code):

- **Firms** (`civ-agents/src/firm.rs`, ADR-0006 §5). A `Firm` is a household workshop: one owner household, a founder, its lines, its own `stores`, posted `offers`, an optional `WageOffer`, books with eight `BookKind`s, and three `Exit`s (`Idle`, `OwnerGone`, `Failure`). It has no legal form beyond the workshop, no licence and no membership. Workshops are few and often idle: in plan §9's NUDGE "hired work is rare" (2026-10-04), every one of nine sickle and hoe workshops was given up.
- **Markets** (`market.rs`). One market per settlement, posted `Offer`s with an `ask_h` in hours of the seller's work, per-market acceptance and no minted money. Any household may offer any good it holds. Nothing gates entry.
- **Trade between settlements** (ADR-0019, `reports.rs`). A household buys elsewhere only by a dated price report (§1). Goods change hands at the seller's door and nothing is in transit (§3), which ADR-0019 calls honest only while there are "no robbery, war, tolls or carts yet". An ask may anchor on replacement from a report (§5), and an `Errand` fetches to resell (§6). The volume is tiny: two years of the M5a demo's year-10 world saw 12 purchases between settlements, all sickles (plan §9, slice AQ step three). ADR-0020 §6 holds "leave to trade" back for "M7's trade policy".
- **Corruption v0 as built** (`population/watch.rs`, ADR-0015 §6). One choice: a watcher who sees a taking may report it, look away, or ask the taker's household for `ask_days` (3) of its food, up to what it can spare. The payment moves on `Channel::Bribe` (code 16, "payment to say nothing"). An unpunished payment lowers the watcher's `risk_seen` (09-09 §1.5's learning). The collector the M4 crime brief §1.8 proposed was not built: each household pays the levy at threshing, with no collector. The storekeeper (`KeepStore`) holds the common store but has no discretion: relief is given by rule (`give_relief` in `population/polity.rs`).
- **Laws** (`polity.rs`). There are 19 `PolicyKind`s and 19 `IssueKind`s. None touches markets, trade or organizations. A body's membership is `Adults`, `Elders`, `Landholders` or `Watch`; its pass rule is `MoreForThanAgainst` or `TwoThirds`; `AmendBody` changes either. An issue opens moves and weighs toward none (ADR-0013 §5).
- **How a household weighs a law** (`Forecasts::gain`, `population/polity.rs`). The forecast is the change in the log of a year's food above `subsistence_share` (0.5), times `w_gain` (10) points. A stance adds what is held dear, talk at the hearth, a view of the other polity, and `w_regard` (1) times regard for the sponsor or the one the law names. Within `stance_margin` (0.25) a member abstains. Attending scores `attend_base` 1 plus `w_attend` 0.5 per point at stake; a quarter of the adults must come; more for than against carries it (content `[polity]`). A sponsor weighs those who regard them at `w_followers` 0.5. Stances are recorded per person and seen by those present (`decisions_seen`): voting is open.
- **Naming.** A sponsor of a store keeper or a watch names "the adult they regard most, themselves at full regard" (`population/polity.rs`). That is favour by regard. Kinship is not a term, and offices state no criteria.
- **Ties** (`ties.rs`, ADR-0014). A `Tie` keeps familiarity, warmth, evidence in four domains, `fear` and `help_h`, a balance of help received less given. Acts include `GiftReceived`, `WagesPaid`, `RentPaid`, `LandLent` and `Traded`. Evidence fades by half in 180 days, warmth in 730 and help in 1,095. The help balance is saved and shown, but no choice reads it; it only decides which ties are kept. Notables deliberate weekly, and the flag grants nothing else (ADR-0014 §4).
- **Factions** (`faction.rs`, ADR-0017 §2). An organization with a founder, an organizer, members, a store on the ledger, `Dues` (channel 17), monthly reviews and a history. It forms only from a grievance against a party (`Blamed`). It is the nearest template for a guild.
- **Values** (`content/core/value`): autonomy, reciprocity and security, which a policy template says it bears on.
- **Scale.** Plan §4.3 sets 3–8k people in 2–3 settlements for M5–M7. Lived villages so far are 18 to about 170 (plan §9). Everything below must work at both sizes.

## Where the reports agree

Eight points recur across 09-09, 09-10, 08-17, 08-12, 08-08, 08-13 and 08-18. I treat them as invariants.

1. **No corruption percentage.** Corruption is "the diversion of particular resources, decisions and enforcement processes—not ... a civilization-wide 'corruption percentage'" (09-09, opening). An opportunity exists only where someone holds discretion or entrusted goods (09-09 §1.2). No person carries a corrupt flag (09-09 §5.1).
2. **Organization is not automatic.** "Do not implement 'wealthy class supports policy' without an organization or other means of coordinating action" (09-10 §1.1). A guild forms when members expect service benefits and privileges to exceed dues, coordination costs and obligations (08-17 §1.1).
3. **Access, attention and decision are separate stages.** "An interest can obtain a meeting but lose the vote" (09-10 §1.3). Money often buys access and effort before it buys agreement (ibid.).
4. **Privileges act through real constraints.** Entry, credit, demand and competition change; there is no "capture increases profits" modifier (09-10 §1.2, §1.6). A monopoly matters "only where it can be enforced" (08-17 §1.5).
5. **A bribe is a transfer.** Social losses come from altered behaviour, and nobody should "add a second generic productivity penalty" (09-09 §1.8; 09-10 §1.6). Every diverted unit has a destination (09-09 §4).
6. **Capture is control of a chain, not a meter.** Ask who can appoint, who can halt a case, who hears appeals (09-09 §5.3). The useful question is "Who can obtain which decisions, through what dependencies" (09-10 §6.3). Durable capture controls reproduction: eligibility, selection, oversight (09-10 §1.5).
7. **Guilds bundle services and restrictions.** Program both, and do not "settle the debate by programming one side's conclusion" (08-17 §1.9; 09-10 §6.3).
8. **Law reclassifies acts without touching goods.** Legality is a function of jurisdiction, date, actor, activity, good, quantity and authorization (08-18 §1.1). Tariffs need officials, records, inspection and route control (08-12 §1.5), and incidence emerges through markets (08-13 §4.1).

**What the reports say to avoid:** fixed interest-group templates (09-10 §5.7, on Victoria 3); The Guild 3 read as evidence (09-10 §5.7; 08-17 §5.7); a population threshold for guilds (08-17 §5.7, on Dwarf Fortress); "a building that grants craftsmanship while penalizing innovation" (08-17, main conclusion); an "oligarchy event" (09-10 §5.5); a capture index in place of its dimensions (09-10 §4).

**Where they differ, or differ from the plan:**

- **Enforcement capacity.** Plan §5.2 has "Capacity = officers × competence × (1 − corruption)". 09-09's opening rejects a corruption scalar. The M4 crime brief §1.7 made capacity a reported measure; I keep that.
- **Drag on discovery.** Plan §5.2 asks for "drag on growth and discovery". 09-09 §2.2 says "Do not reduce every inventor's discovery probability merely because their polity has a high corruption label". Mauro's 0.8 pp of growth is "Low for causal transfer" (09-09 §2.1). §1.9 measures drag and never sets it.
- **Guilds.** Epstein sees training and diffusion; Ogilvie sees exclusion and rents (08-17 §1.9; 09-10 §6.3). Both are to be programmed and compared.
- **Audit priors.** 09-09 §2.3 gives usable evidence 0.40 given an audit; 09-10 §2.2 gives detection 0.50. Both are unestimated priors, and the village has no auditor yet.
- **Relationship memory.** 09-10 §2.2 suggests a 5-year half-life for discretionary relationships (1–20). The kernel's warmth (2 years) and help (3 years) lie inside the range; evidence (180 days) is shorter.
- **What a merchant guild wants.** The plan's demo has merchants pass protection. In TCE a merchant is a household that fetches to resell (ADR-0019 §6), and an import duty taxes its purchases too. 08-17 §1.6 records merchants and producers on opposite sides (Ulm's merchants "supported rural competition"). A merchant guild may want a duty that exempts resellers, or an exclusive right to resell, not a plain tariff (§1.2, open question 5).
- **The demo** names an outcome. 09-10's own example ends "No single outcome needs to be prescribed". §2 treats this as a plot risk, as the war brief treated its siege.
- **"A republic's assembly".** Today's body is a gathering of adults deciding by open acclamation. The republic is the elections brief's; capture must work in either.

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| A trade law (duty, embargo) | Law of an appended kind: goods, settlements, rate, base, exemption, collector if named | A proposal answering an appended issue; each stance |
| A duty owed | ADR-0015 obligation of an appended kind: debtor, good, amount, purchase it arose from, who saw it | A purchase at another settlement's door |
| An organization by stake | ADR-0017's record, its kind and the goods it is about; members; applications and refusals | A founder's scored choice; each member's monthly review |
| A privilege | Law of an appended kind: holder organization, goods, scope, admission rule | A proposal; enforcement by cases |
| Diversion and favour | Ledger channels (appended), sightings and cases as ADR-0015 keeps them | An officeholder's scored choice |
| Patron gifts | Ordinary gifts with a recorded reason; ties as now | The giver's choice |

**Derived, not saved:** every capture diagnostic; each household's forecast of a law; "guild", "monopoly" and "captured" as labels. **Size:** organizations and privileges are few; obligations from duties number about the purchases between settlements.

### 1.1 Who stands to gain: the forecast of a trade law

**Reports.**

- A privilege worth 1,000 grain-units a year gives ten mill owners 100 each and costs 1,000 households one each; the owners "have stronger individual incentives to attend meetings" (09-10 §1.1, an illustration).
- An organization weighs the discounted value of a policy against cash, time and expected sanctions, and must also weigh a productive investment, "Otherwise political spending has no meaningful opportunity cost" (09-10 §5.2).
- A legally liable merchant "may adjust prices, margins, shipment size, or route" (08-13 §4.1).

**My proposal.**

- **Each household forecasts from its own records,** through `Forecasts::gain` as every law is weighed. As a buyer, it loses the duty on what it bought elsewhere last year, or the dearer home price. As a seller of the good at home, it gains the sales it expects back, at its margin. As an exempt reseller, it gains the buyers who no longer fetch for themselves. Its share of the store's takings is what it believes others import, from trips its people saw. Nothing reads another household's records.
- **An illustration of the scale (my arithmetic, with stated assumptions).** A household of five needs about 3.8 million kcal a year (5 × 2,100 × 365). Suppose it holds 1.5 years' need, counting its next harvest, and that an hour of its work is worth about 2,000 kcal (an assumption; the kernel takes each household's own). Then 0.25 points, the stance margin, needs a gain of about 100,000 kcal: about 50 hours a year. Proposing costs 0.5 points, about 100 hours.
- **What that means today (my arithmetic).** A village importing 10 sickles a year at 4 hours each (the M3a demo's asks, as the M5 trade brief reports them) pays 10 hours in all under a 25 % duty (08-12 §2.2's Muziris rate, one regime and not a default). A home smith that won every sale at a 0.8-hour margin (an ask at 1.25 times a 3.2-hour cost) gains 8 hours. Both are far below 50: nobody proposes, and nobody comes for it. That is Olson's result at small stakes (09-10 §1.1), not a fault.
- **Regard then decides.** Full regard for the sponsor is worth 1 point, four times the margin. On a law that moves nobody's food much, stances follow regard for the sponsor. A well-regarded sponsor carries trade laws on regard, which is patronage already, and should be measured before anything is added.

### 1.2 Tariffs and embargoes as laws

**Reports.**

- Charges have different bases: per load, per unit or a share of assessed value. A fixed charge per load "bears more heavily on cheap goods" (08-12 §1.5). There is "no safe universal 'premodern tariff'" (ibid.).
- Collection needs officials, records or witnesses, inspection capacity and control over routes, and in-kind collections become physical stocks (08-12 §1.5). Customs create "route substitution, concealment, bribery, inspection delays, and competition between jurisdictions" (08-13 §1.2).
- Test settings: 0, 2.5, 5, 10 and 20 % of declared value (08-13 §2.3, authored, not estimated).
- Evasion pays "only to the extent that it increases the difference between legal and illicit net returns" (08-18 §1.2). One point of duty went with about 3 % more measured evasion in one setting, a reporting gap and "not a transferable universal elasticity" (08-12 §1.5; 08-18 §2.2).
- A self-reported base is concealed and a third-party one is not: 37 % against 0.3 % (08-13 §2.2, direction only).
- A neighbouring government "can benefit from trade that another government considers illicit" (08-18 §3.2).

**My proposal.**

- **A duty** is an appended `PolicyKind`. Its parameters: the goods, the settlements it applies to, a rate from a content menu, the base (a share of the goods brought home, or a fixed amount a load), an optional exemption (households that offer the good at home, or members of a named organization), and optionally a collector it names, as a watch is named.
- **The taxable event is the purchase.** Goods change hands at the seller's door (ADR-0019 §3), so there is no load to stop on the road. A member who buys at another settlement's door owes the duty to the store as an ADR-0015 obligation. It is paid at home, in kind, as the levy is (ADR-0013 §4). No money is needed.
- **Paying is a choice** (ADR-0013 §3; 08-13 §4.1). The household weighs the duty against the sanction it expects and the chance it believes someone saw. Who saw is real: people of its own settlement at the same door, or a collector posted on the path who sees a carrier come home from that market. Arrears keep their cause (08-13 §4.1).
- **An embargo** is an appended prohibition on buying from, or selling to, people of a named settlement. Keeping it is each person's choice, as a curfew's is, with the duty's case machinery where the law names a sanction.
- **Prices move by themselves.** A household that would owe the duty adds it to the replacement cost in its anchor (ADR-0019 §5). Home asks can then rise toward the duty with no rule naming the other price; incidence emerges (08-13 §4.1).
- **The issue.** An appended `IssueKind`: a household that offers a good at home knows people of its settlement fetched that good elsewhere lately (seen at a door, or told). It opens the duty and embargo moves and weighs toward neither (ADR-0013 §5).
- **The neighbour answers with its own moves.** Its people turned back or charged form views of the polity (ADR-0020 §3) and may hold its own issue. Retaliation, an agreement granting leave to trade (ADR-0020 §6's waiting clause), or nothing all stay possible. No rule makes tariffs answer tariffs.

### 1.3 Guilds: an organization of shared stake

**Reports.**

- A coalition forms when "service benefits + expected privileges > dues + coordination costs + obligations": "private incentives, not a social-welfare test" (08-17 §1.1).
- Keep "association exists", "government recognizes it" and "government enforces its exclusive rights" apart (08-17 §1.1).
- Membership is a ladder of rights: training, working for a member, selling independently, opening a workshop, a mark, voting, transferring (08-17 §1.2). "Record applications, refusals, discouraged potential applicants, admissions" (ibid.).
- Priors for testing: a founding coalition of 3–10 independent workshops, entry 0–180 wage-days, dues 0–10 wage-days a year (08-17 §2.2, "unestimated"). A fixed cost to organize of 30 person-days, tested at 5–300 (09-10 §2.2).
- Conflicts run within the guild: exporters against local sellers, employers against employees, rich officers against poorer members (08-17 §1.6). Officers propose and members support by their own forecasts; there is no "guild welfare" score (08-17 §5.3).
- A guild may also be a network across towns, not a single firm (08-12 §1.3, the Hansa). The treasury is explicit, and claims can outrun dues (08-17 §1.8).

**My proposal.**

- **Generalize ADR-0017's faction** so an organization may form around a shared stake as well as a grievance. The founder is a household member who sells a good at home or fetches it to resell. They weigh the forecast of a privilege or duty they would seek, plus what membership gives (v0: mutual relief from the treasury, as factions already allow), against the fixed cost and the dues.
- **The search is sparse.** A founder asks those they know who sell the same good, at home and in a neighbour village where they trade. At village scale a good may have one to four sellers, at or below 08-17's search floor of 3, so the founding count is a tuning value and a guild of two is allowed.
- **Members may live in several settlements.** A privilege is still granted by one polity, for its own settlement.
- **The guild's own rules are values** over authored vocabulary, as a body's are: who is admitted (anyone who sells the good, by sponsorship of a member, or by a fee), dues, and who decides (the organizer, or the members by more for than against). Each application, refusal and admission is recorded.
- **Recognition is a law.** A law of an appended kind names the organization. Existing, recognized and privileged are three states.
- **Its program is the organizer's choice,** weighed by members' own forecasts. Makers and resellers may split over a duty, and that split is a result.

### 1.4 Privileges: licences and monopoly

**Reports.**

- Specify what an exclusive right covers: occupation, product, process, place, customer or jurisdiction (08-17 §1.5). E[consequence] = P(detection) × P(sanction upheld | detection) × sanction (ibid.).
- The excluded can become an employee, move, sell through an intermediary, change products, petition, conceal or abandon the trade (08-17 §1.5).
- "A license can be an asset owned separately from skill, labor, and real estate" (08-17 §3.5, on Ottoman gedik).
- Entry eligibility is a charter dimension: open, licensed, mastership required, chartered privilege (08-08 §1.2). A guild workshop stays independently owned (08-08 §1.3).

**My proposal.**

- **A privilege law** grants a named organization the sole right to offer named goods at doors in the settlement, or the sole right to resell goods fetched elsewhere (a staple). Admission follows the organization's rule.
- **Detection is easy and sanction is not.** An offer is posted at a door, so every buyer sees it. A breach becomes a case only if someone brings it, and the gathering hears it by ADR-0015's existing procedure, where regard for the accused weighs. Courts, when they come, change only that step.
- **The barred respond through existing moves:** hired work for a member (`WageOffer` exists), selling through a member, selling only to those who will not tell, moving (ADR-0018), a petition (ADR-0017 §3), or giving up. A workshop that closes because it was barred gets an appended `Exit`, apart from `Idle`.
- **Holding a licence is not knowing the craft** (08-17, main conclusion). v0 keeps a licence personal to the organization's member; transfer and lease wait.

### 1.5 Lobbying: how an interest reaches the votes

**Reports.**

- Lobbying can be "a subsidy to legislators who already share some of the lobby's objectives" (09-10 §1.3). Officials have limited time; a prior of 4 organized meetings a month, tested 1–20 (09-10 §2.2).
- Revealed donor status raised access to senior officials 3–4 times, with no measured effect on votes (09-10 §2.1).
- An official's utility separates public benefit, constituency, household, norms, career and sanctions on one scale (09-10 §5.3).
- Clientelism is contingent reciprocity, imperfectly observed: "Secret voting must not become perfectly visible to patrons" (09-09 §1.7). Benefits can be "food during shortages, legal protection, tenancy, employment" (ibid.).

**My proposal.** A gathering has no meetings to buy, so each stage maps to what exists.

- **Access is proposing.** Anyone may propose (ADR-0013 §1). Notables weigh moves weekly and others when something reaches them (ADR-0014 §4), so a regarded master proposes more often. Nothing new is needed; measure who sponsors what.
- **Attention is talk.** Members talk at the hearth, and talk already moves stances (M4c slice AG's opinion points).
- **The decision has three levers, all existing terms:** members' stakes bring them to the gathering (`w_attend`); diffuse losers abstain within the margin; regard for the sponsor tips the rest (§1.1).
- **Patron gifts** are ordinary gifts from a member's household or the treasury, with the giver's reason recorded ("before the gathering on the duty"). They write `GiftReceived` and the help balance as any gift does.
- **Contingent reciprocity needs a new term:** what a member believes they owe the sponsor, read from the help balance that nothing reads now (open question 7). A patron who saw a client oppose may stop giving. Under open acclamation the patron sees every stance; a secret vote, if the elections brief adds one, hides it.
- **Spending competes with production.** Gifts come out of food the household or treasury would otherwise keep, so the forecast sees the cost (09-10 §5.2).
- **Bribing a member of an elected assembly** is a collusive bribe (09-09 §1.1), weighed by 09-09 §1.3's ΔU and exposed by witnesses. It waits for the elections brief's assembly.

### 1.6 Corruption beyond the watch

**Reports.**

- ΔU = B + K − C − M − p̂(F + V): private benefit, kin and clients, arranging costs, moral cost, perceived punishment and the value of the office lost (09-09 §1.3). Citizens may "pay, refuse, complain, seek another provider, evade, delay or migrate" (ibid.).
- p(punishment) = p(exposure) × p(proof | exposure) × p(sanction | proof); exposure may come from "a witness, a rival, a discrepancy or a public failure" (09-09 §1.4).
- Start with "a granary steward, a tax collector and a judge" (09-09 §5.4). In a granary of 100, if 12 are stolen and 3 spoil, 85 remain, and the 12 still exist somewhere (09-09 §1.8).
- Priors, unestimated: a diversion ceiling of 0.15 of the flow, an upward patron payment of 0.20, an old-evidence half-life of 1 year (09-09 §2.3).
- "An office has operating costs as well as a salary"; forbidding fees without funding drives them underground (09-09 §3.4).
- Nepotism is a distortion "relative to the institution's stated criteria"; "A relative is not necessarily incompetent" (09-09 §1.1).

**My proposal.**

- **The steward.** The store's keeper may draw from the store for their own household, a choice weighed by ΔU with the keeper's objection, risk and kin. It moves on an appended `Diverted` channel, up to a ceiling. Exposure is a public failure (relief refused while people believe the store holds food) or someone who saw. The answer is a case or a petition against the office (M4c slice AH), as now.
- **Relief by favour.** When the store cannot answer every ask, the keeper chooses whom first, by regard. The order is recorded, so favour is visible in the truth layer.
- **The collector.** A duty's collector, if named, sees purchases on the path and may under-assess for a payment (collusive) or ask one (extortionary), on the `Bribe` channel. The payer may refuse, pay or tell.
- **Favour in naming** stays as built: the sponsor names whom they regard most. Nepotism is then measured, not added: the share of nominees who are close kin of the sponsor against the share among those eligible.
- **Reinforcement comes only from mechanisms** (09-09 §1.5). Learning is built. Protection comes from the gathering hearing cases with regard, so an officeholder with many clients is found less often. Recruitment comes from naming by regard. Nothing else makes corruption persist.
- **Office pay waits for money.** Offices are unpaid, so informal takings are plausible (09-09 §3.4). Paying them is a sibling brief's question.

### 1.7 Capture and closure

**Reports.**

- Capture becomes durable when an interest controls eligibility, selection, oversight and economic reproduction (09-10 §1.5). Closure is ordinary: "Require sponsorship by two existing members", "Exempt current license holders", "Reserve specified offices for members of recognized households", "Give the council authority to nominate its successors" (09-10 §5.5).
- Venice's closure took about 26 years (1297–1323), and commoners' share of surviving trade contracts fell from about 51 % to 27 % to 3 % (09-10 §2.1, small archival samples).
- Office-sharing can stabilize government and exclude outsiders at once (09-10 §3.2, Dutch regents). Florence's 1378 upheaval was followed by an oligarchic restoration in 1382 (ibid.).
- Reversal comes from outsider organization, alternative finance, new opportunities, independent investigation or the end of privileges, and "Reversal is not synonymous with replacement" (09-10 §1.7, §6.3).

**My proposal.**

- **Closure is an amendment.** `AmendBody` gains appended membership rules: members of a named organization, or holders of a privilege. A licence's admission rule is the organization's own and can be changed only by its own decision.
- **Each step is a law someone proposed,** with every stance kept. A captured body is a history of such steps, never a state.
- **Reversal uses existing moves:** a faction against the organization (a grievance at a refused admission is appended to what can be `Blamed`), a petition, repeal, a revolt and a founding window (ADR-0017 §4–5), or leaving (ADR-0018).
- **Capture is a derived diagnostic** (09-09 §5.3; 09-10 §4): the share of laws passed whose sponsor belongs to one organization; the share of stances that carried them made by members and by those who owe them; who names officeholders; whether cases against members proceed; refusals of admission; the share of trade with other settlements done by non-members. The observer says, for example, "the gathering passed the duty on sickles; most who stood for it work for or owe the sickle-makers".

### 1.8 Drag on growth and discovery

**Reports.**

- Corruption reaches the economy through public services, enforcement, investment, entry, human capital, discovery and diffusion, and political stability (09-09 §1.8's table).
- Connected Italian firms grew in employment and revenue "without corresponding productivity growth" (09-09 §2.2; 09-10 §1.6).
- Let corruption change funding, skills, entry and returns, and distinguish "inventing a technique from getting permission, capital or materials to deploy it" (09-09 §2.2).
- Guilds can affect invention, adoption and diffusion differently; secrecy should "not make knowledge magically invisible" (08-17 §1.7).

**My proposal.** No multiplier. Drag is what these mechanisms do, and it is measured:

- a household that knows how to make a barred good cannot sell it, so it makes less, practises less and gains less skill (ADR-0006 §2);
- a protected workshop avoids the `Idle` exit without its own cost per unit falling;
- hours go to gatherings, gifts and dues instead of fields;
- buyers pay more or walk further.

Measures: applications and refusals; workshop survival against own cost per unit; prices of protected goods against the neighbour's; hours and goods spent on influence; techniques known against techniques practised for sale. Restricting who a member may teach waits (§3).

## 2. Plot guards

- **No authored guild.** Content supplies the vocabulary (organization kinds, privilege and duty templates, menus of rates). No merchant guild, interest group or faction is seeded.
- **No issue weighs toward a duty, an embargo or a privilege** (ADR-0013 §5). "Imports rose" is an issue that opens moves, never a cause.
- **No meter.** No corruption rate, capture index, guild productivity modifier or discovery penalty enters any choice (09-09 §5.3; 08-17 §1.9; 09-09 §2.2).
- **No oligarchy event and no era.** Closure is a run of amendments (09-10 §5.5).
- **Truth stays apart.** A diversion, a bribe or a purchase is known only by those who saw it or heard (ADR-0015 §1). Test that an unseen payment changes nobody's forecast.
- **Do not tune to pass.** §1.1's arithmetic says protection is not worth proposing at today's trade volumes. Raising `w_gain` or lowering the margin until a tariff passes would be plot. If the demo has not happened after a time-boxed week, nudge a prior (an organization's fixed cost, a report's half-life) and log a `NUDGE:` (plan §1, rule 3). Never author an event, and say so if the demo does not arrive.
- **The demo's outcome is one of several.** 09-10's example lists consumers organizing, a rival financier, revocation, incumbents adopting the technique, or the outsider leaving.

## 3. What to defer, and why

| Defer | To | Why |
|---|---|---|
| Apprenticeship contracts and teaching restricted to members | After v0 guilds | 08-17 §1.3 needs contracts with terms and exits; ADR-0008's teaching has no permissions yet |
| Quality, marks and inspection | When goods carry quality | 08-17 §1.4 separates actual quality, belief and conformity; goods have no quality attribute |
| Office purchase and tax farming | With money and credit | 09-09 §3.2–3.3: prices and advances need a medium and debt |
| Connected lending | With the credit brief | 09-10 §2.1's 45 % more borrowing needs lenders |
| Cumulative tolls along routes | With a goods-in-transit holder | 08-12 §1.5; ADR-0019 §3 has nothing in transit |
| Oversight chains and audit agencies | With courts | 09-10 §3.2 (colonial Peru) and 09-09 §3.6 need appointed overseers and forums |
| Smuggling organizations and protection | After duties exist | 08-18 §1.5–1.6: organization needs a reason; let single evaders come first |

## 4. Risks and validation

**Too little trade to fight over.** Twelve purchases in two years (plan §9) make every trade law worth hours. A world of 3,000 in three villages may trade more, and the M5 trade brief's carry to sell (§1.2, step 4) would put outsiders' goods at home doors. Measure the volume and the forecast distribution before building organizations.

**Regard swamps stakes.** With stakes under the margin, whoever is most regarded carries any trade law. That may be plausible patronage or an artefact of `w_regard`. Report stance receipts by term before judging.

**Compute.** A world with no organization, duty or privilege must give the same digests as the build before. Organizations review monthly (09-10 §5.6; 08-17 §5.5). A duty adds one obligation a purchase between settlements. Add duties paid, diversions and bribes to Gate B (ADR-0011 §5) with wide tolerances.

**Stylized facts and tests** (plausibility checks, never targets):

- **Olson.** Hold the total stake of a privilege fixed and vary how many share it: concentrated stakes should propose and attend more (09-10 §4).
- **Access is not success.** More proposals by an organization need not mean more laws passed (09-10 §4).
- **Revenue and predation together.** A collector who keeps more can raise both what reaches the store and bribes, as in Pakistan, where revenue rose 9.4 log points and reported bribe rates about 32 % (09-09 §2.1; direction only).
- **Monitoring reduces, never ends.** In Olken's roads, universal audit cut missing expenditure by about 8 pp and about 20 % stayed missing (09-09 §2.1). A village watching its store should still lose some.
- **Corruption moves.** Watch the store and leave naming unwatched: favour should shift, not vanish (09-09 §4).
- **Protected, not productive.** Protected workshops survive longer without lower costs (09-09 §4; 09-10 §4).
- **Evasion rises with the rate** (08-12 §1.5; direction only). A cut from 119 % to 12.5 %, as Britain's tea duty in 1784, should move trade back to the lawful channel through the same households (08-18 §3.2).
- **Representation is not inclusion.** Count members, the barred and the discouraged together (08-17 §4).
- **Kin among entrants.** New masters who were masters' children were 26–44 % in selected European samples (08-17 §2.1): a guild should neither be wholly hereditary nor ignore kin.
- **Removal has costs.** Repealing a privilege can disrupt before entry helps (08-17 §3.6, Japan 1841).
- **Matched runs** (08-17 §5.6; `civ-host twin`): no organizations; organizations without privileges; with privileges; with duties.
- **Conservation.** Every diverted unit, duty and gift has a destination (09-09 §4). Run once with reinforcement switched off to see what makes corruption persist (ibid.).

**Observer.** An organization's page: members, rules, treasury, applications and refusals, laws it sponsored. A law's stances with each person's terms in words, including what they owe the sponsor. A store's accounts as paid in, kept, diverted and given (08-13 §1.1). Capture as sentences, never a percentage (09-09 §5.3).

**Expensive to reverse.** Plan §1 allows about three ADRs a milestone, and elections, courts and credit will want theirs. I suggest one ADR, "organizations, privileges and trade policy": organizations by stake (amending ADR-0017 §2); privilege and duty laws, their obligation kinds and channels (`Diverted`, a duty channel); the barred exit (amending ADR-0006 §5); the purchase as the taxable event (keeping ADR-0019 §3); and leave to trade as a clause (ADR-0020 §6). Corruption beyond the watch fits ADR-0015 §6 without a new ADR.

## Suggested slices

1. **Trade laws first:** the duty and the embargo as laws, the appended issue, obligations from purchases, and each household's forecast, measured in lived worlds before any organization exists.
2. **Organizations by stake:** ADR-0017's record founded from a shared stake, with rules, dues, applications and the recognition law.
3. **Privileges and the barred:** the licence and the staple, cases for breaches, the barred exit, and the drag measures.
4. **Corruption and patronage:** the keeper's diversion, relief by favour, the collector, patron gifts and the owed term, with the capture diagnostics.
5. **The demo,** with the elections brief's assembly, time-boxed, its stances told in words.

## Open questions for the designer

1. **The taxable event.** Is it the purchase at the other settlement's door, known by who saw it, as proposed? Or a load on the road, which needs ADR-0019 §3's goods-in-transit holder?
2. **Paying duties.** In kind to the store, as the levy is, until the credit brief brings money?
3. **Organizations.** A new kind of ADR-0017's faction, as proposed, or a record of their own?
4. **Across settlements.** May a guild have members in two villages while each polity grants its own privileges?
5. **Exemptions.** May a duty exempt an organization's members? It gives merchants a reason to want protection, and it is vocabulary a sponsor chooses, not a plot. Is that line safe?
6. **Breaches.** Does the gathering hear a breach of a privilege, as proposed, or the organization's own officer, until courts come?
7. **What a client owes.** Should the help balance enter stances? It changes every world's votes and needs a dashboard rerun.
8. **Open voting.** Acclamation lets patrons see every stance. Keep it, and leave secret voting to the elections brief?
9. **The keeper's discretion.** Add relief by favour, which changes who eats in a lean year?
10. **What counts as the demo.** If the world's merchants seek a staple or an exempting duty rather than a plain tariff, does that meet "protectionist tariffs"?
11. **Trade volume.** Is a world of 3,000 the route to enough trade, or carry to sell, or both?
12. **Nepotism.** Is the kin share among nominees against the kin share among the eligible enough, given that no office states criteria?
