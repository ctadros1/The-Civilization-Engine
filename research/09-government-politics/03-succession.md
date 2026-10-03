# Succession rules and succession crises

## A simulation-ready report for The Civilization Engine

**TCE should model succession as a transfer of recognized authority, organizational control, and political obligations—not simply as replacement of the person occupying an office.** A rule can identify an undisputed heir while leaving that heir unable to command the army. Conversely, competing claims can be resolved peacefully through election, compensation, regency, or power-sharing.

The strongest quantitative evidence supports three distinctions: **deposition during a reign is not the same outcome as civil war around succession; peaceful departure does not guarantee peaceful accession; and survival of a regime does not imply an uncontested transfer.** The studies below measure these separately. [Studocu](https://www.studocu.com/row/document/univerzita-j-selyeho/historia/09a-kokkonen-and-sundell/10166680)

There is useful comparative evidence for European primogeniture, election, and agnatic seniority, and for succession arrangements in modern African autocracies. The reviewed literature does **not** supply a defensible, globally comparable seven-category table of crisis probabilities covering every rule in the question. Where evidence is missing, TCE should calibrate underlying mechanisms rather than invent historical percentages.

---

## 1. Mechanisms: what a succession system actually does

### 1.1 Separate the outcomes before implementing the rules

Use the following as **TCE operational definitions**. They are deliberately separate flags rather than mutually exclusive event types.

| Outcome | Suggested simulation definition |
| --- | --- |
| Routine transfer | The prescribed procedure completes, and the incoming authority obtains effective control without organized resistance. |
| Contested succession | Politically consequential actors recognize incompatible claimants or incompatible procedures. Violence is not required. |
| Irregular transfer or usurpation | Someone acquires authority outside the previously operative procedure. Different actors may disagree about its legality. |
| Deposition | An incumbent is involuntarily removed. Record whether removal was constitutionally authorized, a coup, rebellion, or foreign intervention. |
| Succession civil war | Organized domestic armed groups fight over supreme authority, or over political arrangements directly precipitated by the transfer. |
| Dynastic change | The ruling lineage changes, whether peacefully or violently. |
| Regime change | The institutions governing political authority change substantially. |
| Fragmentation | Formerly subordinate territories become independently governed. This may be negotiated rather than violent. |

A regency is a **delegation or division of authority**, not necessarily a succession. Similarly, the death of a ceremonial sovereign need not replace the effective executive.

For logging, retain the incumbent’s departure, successor selection, accession, recognition, and consolidation as separate timestamps. A transition can appear peaceful on accession day but break down months later.

### 1.2 Build succession from independent institutional components

Instead of an enum containing only `Primogeniture`, `Election`, or `Tanistry`, compose each rule from:

| Component | Questions the institution must answer |
| --- | --- |
| Eligibility | Which kin, lineages, sexes, ages, citizens, officeholders, religious members, or generation classes qualify? |
| Priority | Does birth order, genealogical branch, seniority, nomination, rotation, or voting determine precedence? |
| Selector | Who nominates, votes, confirms, adjudicates, or vetoes? These can be different bodies. |
| Timing | Is a successor designated in advance, selected after vacancy, or chosen on a fixed calendar? |
| Activation | Does authority transfer at death, proclamation, oath, election certification, coronation, or another act? |
| Contingencies | What happens during minority, incapacity, disappearance, disputed parentage, or failure of an eligible line? |
| Enforcement and amendment | Who interprets the rule, sanctions violations, and can change it? |

This decomposition accommodates historical hybrids. Malaysia’s federal monarchy, for example, combines a restricted set of eligible rulers, an ordered nomination list, voting, acceptance by the candidate, and a five-year term. It is not adequately represented by either “hereditary” or “elective” alone. [Majlis Raja-Raja](https://www.majlisraja-raja.gov.my/en/election-of-his-majesty-yang-dipertuan-agong)

### 1.3 The requested rule types

The implementation column below is a proposed abstraction, not a claim that every society using the label followed identical procedures.

| Rule | Core principle and implementation | Principal advantages and failure paths to represent |
| --- | --- | --- |
| **Primogeniture** | Rank genealogical branches by birth order; specify sex preference/exclusion and whether descendants represent a deceased parent’s branch. | Produces a clear focal heir, but can select a child, an incapable person, or someone unacceptable to powerful groups. Extinction of the preferred branch exposes contingent rules. |
| **Lateral succession / agnatic seniority** | Prioritize brothers or another senior generation before descendants. Distinguish seniority of *branches* from the age of individual dynasts. | Usually supplies adult successors, but can produce short reigns and several near-equal, already powerful candidates. |
| **Tanistry** | For a Gaelic-style template, select an eligible dynastic successor, often during the incumbent’s lifetime, rather than automatically taking the eldest child. Make the eligible kin group and selecting body explicit. | Can select an experienced adult and balance branches. Rival selections, exclusion of branches, and an incumbent’s attempt to favor a child can undermine the settlement. |
| **Elective monarchy** | An authorized body chooses a monarch, usually for life. Hereditary eligibility can coexist with election. Define quorum, voting or consensus procedure, vetoes, and confirmation. | Can accommodate elite preferences. Rival assemblies, disputed electors, outside sponsorship, or incompatible bargains can produce competing “valid” elections. |
| **Rotation** | An ordered house, clan, locality, or generation class gains its turn; another procedure selects the person or governing group within it. | Distributes access predictably. Disputes concern skipping a turn, class membership, eligibility, or refusal to relinquish office. |
| **Appointment / designation** | An incumbent or authorized body names a successor. Specify revocability, publicity, ratification, and whether appointment itself grants powers. | Can identify a capable successor early. A powerless nominee may be ignored; an empowered nominee can become a competing center of authority. |
| **Election to a term-limited or accountable office** | Define franchise, candidate eligibility, counting, adjudication, term, reelection rules, and the authority that enforces the result. | Regular opportunities to compete can make losing tolerable. Breakdown occurs when defeat threatens survival, results lack credibility, or coercive organizations reject them. |

The European distinction between primogeniture and seniority is documented in the comparative monarchy literature. Gaelic tanistry requires particular caution: its interpretation was entangled with English legal intervention, and it should not be conflated with either property partition or the broader comparative label “bloody tanistry” sometimes applied to Eurasian dynastic competition. [Studocu](https://www.studocu.com/row/document/univerzita-j-selyeho/historia/09a-kokkonen-and-sundell/10166680)

**Keep inheritance of private property, succession to political office, and partition of territory separate.** They may be linked by a society’s institutions, but one does not logically entail the others.

### 1.4 Implementable causal mechanisms

#### Coordination: people need a shared expectation, not just a written answer

A succession rule can make one candidate the expected choice of everyone else. Designation, public recognition, and confirmation help create that expectation; even medieval designated heirs could require substantial political acceptance. [Cambridge University Press](https://www.cambridge.org/core/books/abs/paths-to-kingship-in-medieval-latin-europe-c-9501200/unanimity-and-probity/A4DEE8366EF397A1860A7756502C946B)

**TCE rule:** actors evaluate both their preferred claimant and whom they expect other actors to support. Public oaths, assemblies, proclamations, and ceremonies update these expectations. Information spreads through existing communication networks, rather than reaching every province instantly.

This permits a useful emergent distinction: *ambiguous law but broad agreement* can be stable, while *clear law but organized rejection* can be dangerous.

#### Commitment: accepting defeat must be safer than fighting

Fearon’s model of self-enforcing democracy explains why an electoral calendar and public rules can coordinate resistance to violations, but also why an armed incumbent may reject defeat. Future opportunities to win can help sustain compliance. Egorov and Sonin’s succession model emphasizes how treatment of defeated rivals changes subsequent contenders’ expectations. [OUP Academic](https://academic.oup.com/qje/article-abstract/126/4/1661/1923169)

**TCE rule:** a losing claimant compares resistance with a settlement offering some combination of physical safety, property, office, autonomy, and future access to power. Promises are discounted according to enforceability and the winner’s reputation.

Do not make assassination or execution automatically optimal. Sparing a rival can preserve a dangerous claimant, but it can also make future peaceful surrender credible.

#### Successor capacity: nomination and accession are different problems

Meng’s research distinguishes identifying a successor from empowering that successor to take office. The relevant support includes organizational position and resources, not merely being named in a constitution. [Anne Meng](https://www.annemeng.com/uploads/5/6/6/6/56666335/jcr_got.pdf)

**TCE rule:** track a successor’s actual connections to administrators, treasury officials, guards, military commanders, party organizers, or clan authorities. An appointment adds a claim and perhaps specified powers; it does not automatically transfer everyone’s loyalty.

Incumbents must face a real tradeoff: strengthening a successor can improve the eventual handover while creating a rival power center before the incumbent leaves.

#### Kinship: relatives are potential allies as well as claimants

Research covering 27 European monarchies finds that larger royal families generally protected incumbents against deposition; effects differed by relatives’ sex and the source of the challenge. A rule that adds rebellion risk for every additional sibling would therefore miss an important observed pattern. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/715065)

**TCE rule:** generate both claims and relationships from kinship. Relatives may provide trusted administrators, marriage alliances, military support, and continuity. Dangerous competition depends on **rival claims combined with independent backing**, not merely family size.

#### Regency: preserve the office while bargaining over its powers

Xiong’s study of Chinese emperors associates minority rule with political and fiscal deterioration, with institutional variation in its effects. It supports treating minority as a potential vulnerability, not assigning one universal penalty to every child ruler or every regent. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0147596724000453)

**TCE rule:** separate the legal holder from the people exercising appointment, taxation, military, judicial, and diplomatic powers. A regent’s interests depend on kinship, expected tenure, accountability, and what happens when the ruler comes of age.

The end of a regency should itself be a possible bargaining episode. Returning powers may be routine; it may require guarantees; or an entrenched regent may resist. Do not impose an intrinsic instability modifier for a female regent.

#### Organizational continuity: succession need not stop government

**Recommended mechanism:** determine which operations require the departing ruler personally. Standing tax assignments, court procedures, delegated military commands, and multiyear contracts can continue. Decisions that require personal authorization may stall.

This produces institutional variation without a blanket “new ruler: −20% productivity” modifier. A highly centralized executive bottleneck and an administratively delegated polity respond differently to the same weak successor.

#### External intervention: domestic uncertainty creates opportunities

The European succession-and-war research also finds increased interstate-war risk around a monarch’s natural death. The evidence does not establish that the same succession arrangements moderate domestic and foreign conflict identically. [Göteborgs universitet](https://www.gu.se/sites/default/files/2020-05/2017_9_Kokkonen_Sundell.pdf)

**TCE rule:** neighboring governments can reassess invasion, tribute demands, claimant sponsorship, or recognition after a succession. They act on their interests, intelligence, and available forces—not on a scripted “succession war” event.

---

## 2. Parameters and quantitative calibration

### 2.1 Reading the evidence correctly

The tables distinguish **observed proportions**, **model-predicted probabilities**, and **institutional rules**.

Confidence refers to usefulness as a historical calibration target:

* **High:** a clearly documented institutional provision.
* **Medium:** a useful comparative estimate, but dependent on sample, coding, and identification assumptions.
* **Low:** insufficient evidence for transferring a numerical value beyond its particular context.
* **Design:** an engineering choice, not an empirical estimate.

None of the reported differences should become an unconditional causal multiplier attached to a rule label.

### 2.2 Deposition and tenure by succession rule

Kokkonen and Sundell’s final 2014 study covers **961 monarchs in 42 European states, 1000–1800**. Its descriptive results are: [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/delivering-stabilityprimogeniture-and-autocratic-survival-in-european-monarchies-10001800/2399079C174599A840E5230E8827609C)

| Rule category | Monarchs | Deposed under authors’ broad coding | Deposed under narrower *Dynasties of the World* coding | Mean tenure, approximately | Confidence |
| --- | --- | --- | --- | --- | --- |
| Primogeniture | 451 | **16% per reign** | **8% per reign** | **21 years** | Medium |
| Election | 386 | **49% per reign** | **21% per reign** | **12 years** | Medium |
| Agnatic seniority | 124 | **57% per reign** | **43% per reign** | **9 years** | Medium |

The broad measure includes domestic murder, forced abdication, and death in civil war. These are **whole-reign outcomes**, not probabilities that a succession immediately causes war. The two coding columns are alternative definitions, **not confidence-interval bounds**. [Studocu](https://www.studocu.com/row/document/univerzita-j-selyeho/historia/09a-kokkonen-and-sundell/10166680)

For TCE, these are scenario-level validation targets. Differences in age, institutional setting, historical period, and selection into succession systems prevent treating the raw percentages as portable rules.

### 2.3 Civil-war risk around natural-death succession

The following numerical estimates come from the **2017 working-paper version**, *The King Is Dead*, preceding the published article *Leader Succession and Civil War*. That version examines 28 polities over 1000–1799, with 13,575 country-years. Keep its version identifier attached to the numbers. [Göteborgs universitet](https://www.gu.se/sites/default/files/2020-05/2017_9_Kokkonen_Sundell.pdf)

| Condition | Predicted civil-war onset probability | Unit | Interpretation | Confidence |
| --- | --- | --- | --- | --- |
| Primogeniture, without the natural-death succession shock | **2.7%** | Country-year | Model comparison baseline | Medium |
| Primogeniture, natural-death succession year | **6.3%** | Country-year | Elevated risk despite the clearer rule | Medium |
| Other succession systems, without the shock | **2.7%** | Country-year | Model comparison baseline | Medium |
| Other systems, natural-death succession year | **14.9%** | Country-year | Larger estimated succession-associated increase | Medium |

These are adjusted model predictions, not raw percentages of all successions. They concern **onset**, not the fraction of years already experiencing civil war. The published research supports the broader finding that primogeniture moderates succession-associated civil conflict. [Göteborgs universitet](https://www.gu.se/sites/default/files/2020-05/2017_9_Kokkonen_Sundell.pdf)

**Implementation warning:** do not convert the 14.9% figure into a daily rebellion roll, and do not apply it on top of an agent model already generating the same conflict mechanisms.

### 2.4 Broader comparative anchors

| Parameter or finding | Value | Population and unit | What it can calibrate | Confidence |
| --- | --- | --- | --- | --- |
| Peaceful authoritarian leadership transitions | **33/119 = 27.7%** | Meng’s selected Sub-Saharan African transitions, 1960–2010; per transition | Combined incumbent departure and successor accession | Medium |
| Nonpeaceful incoming transition despite peaceful incumbent exit | **19/52 = 36.5%** | Same sample; conditional on peaceful departure | Why departure and accession need separate stages | Medium |
| Predicted peaceful transition with versus without a vice-president succession rule | **67% versus 11%** | Same study; model-predicted probabilities | Empowered succession arrangements, not a universal “VP bonus” | Medium |
| Regime survives one year after dictator dies in office | **87%** | Kendall-Taylor and Frantz: 79 deaths, 1946–2012 | Regime continuity after leader death—not peaceful succession | Medium |
| Mean reign under father-to-son succession versus other arrangements | **Just over 15 versus about 6 years** | Smith’s dataset of over 300 Eurasian dynasties, 1000–1799 | Broad tenure patterns; father-to-son is wider than primogeniture | Medium |
| Oromo gada leadership rotation | **8 years** | Rotation interval in the documented system | An authored institutional term | High |
| Malaysian federal monarch’s term | **5 years** | Constitutional term | A hybrid elective-rotational template | High |

Sources: Meng’s final 2021 article; Kendall-Taylor and Frantz’s *When Dictators Die*; Smith’s *Storm from the Steppes*; UNESCO’s gada documentation; and Malaysia’s Conference of Rulers. [Anne Meng](https://www.annemeng.com/uploads/5/6/6/6/56666335/jcr_got.pdf)

Meng’s “nonpeaceful” category must not be relabeled “civil war.” It incorporates the study’s rules for irregular departure or accession and successor survival. Likewise, the 67%–11% comparison is an observational model result, not evidence that copying an office title produces that effect everywhere. [Anne Meng](https://www.annemeng.com/uploads/5/6/6/6/56666335/jcr_got.pdf)

### 2.5 Where rule-specific rates remain unknown

| Rule or institution | Defensible numerical conclusion from the reviewed evidence |
| --- | --- |
| Gaelic tanistry | No harmonized crisis-rate denominator established here. Do not substitute the European “election” or “seniority” rate. |
| Rotation generally | Specific terms and procedures are documented; a pooled historical breakdown rate is not. |
| Appointment generally | Modern studies support conditional effects, but appointment to a royal heirship, vice-presidency, military command, and party leadership are not interchangeable treatments. |
| Democratic election generally | Requires a specified population and outcome: disputed results, violence, refusal to concede, coup, and regime breakdown are different measures. |
| Regency generally | Effects depend on minority, delegation, coalition structure, and context; no universal annual usurpation rate is justified. |
| Forager or earliest farming leadership replacement | Evidence does not support a global quantitative succession-crisis rate. |

### 2.6 Proposed engineering parameters—not historical estimates

Use these to make a first implementation testable. They should remain visibly separate from empirical content packs.

| Parameter | Starting value or test range | Unit | Role and provenance |
| --- | --- | --- | --- |
| Outcome observation windows | **30, 90, 365** | Days after activation | Log immediate recognition, consolidation, and delayed breakdown. **Design.** |
| Coalition review during active crisis | **1–7** | Days | Scheduling cadence; urgent events trigger immediate review. **Design.** |
| Review during routine politics | **30–90** | Days | Avoid daily reevaluation of every relationship. **Design.** |
| Detailed active claimant shortlist | Start at **8**; test **4–16** | Candidates per contested office | Performance budget, not a legal limit. Preserve other claims lazily. **Design.** |
| Believed credibility of a guarantee | **0–1** | Probability-like belief | Endogenous to enforcement and reputation; no universal initial value. **Design.** |
| Political majority threshold in synthetic test societies | Test **12–21** | Years of age | Sensitivity exercise only; actual scenarios use their authored rule. **Design.** |
| Recognition and coercive-support measures | **0–1**, separately | Shares | Never collapse accepted legitimacy and military control into one variable. **Design.** |

---

## 3. Variation across eras and world regions

**Do not use technology eras as succession-law unlocks.** Organize variation around kinship, military organization, political scale, administrative delegation, selector membership, and accepted procedures.

### 3.1 Foragers and early farming societies

Many small-scale societies do not possess a single permanent office of supreme power. Research on 23 explicitly egalitarian hunter-gatherer societies examines leadership associated with teaching and cultural transmission; that sampling frame does not represent every foraging society, especially more stratified cases. [Nature](https://www.nature.com/articles/s41467-025-58764-9)

For such TCE societies, leadership can be **task-specific and overlapping**: hunting coordination, mediation, ritual knowledge, or external negotiation. Replacement can occur when others stop following one person, without a formal vacancy or enthronement.

Contemporary ethnography should not be treated as a direct recording of prehistoric institutions. For the earliest farming communities, the reviewed evidence is particularly thin on named successions and comparable crisis denominators. A settlement’s agricultural economy alone should not determine whether leadership is hereditary, elective, collective, or informal.

### 3.2 Pre-industrial institutions were highly diverse

| Region or case | Historically important distinction | Consequence for TCE |
| --- | --- | --- |
| **Medieval Latin Europe** | Designation and hereditary claims could coexist with confirmation, elite acceptance, and judgments of fitness. | “First in line” and “accepted ruler” remain separate states. [Cambridge University Press](https://www.cambridge.org/core/books/abs/paths-to-kingship-in-medieval-latin-europe-c-9501200/unanimity-and-probity/A4DEE8366EF397A1860A7756502C946B) |
| **Gaelic Ireland** | Tanistry and its replacement were entangled with competing legal systems and colonial intervention. | Succession-law change can be imposed or negotiated; it is not simply an efficiency upgrade. [Cambridge University Press](https://www.cambridge.org/core/books/abs/sir-john-davies-and-the-conquest-of-ireland/cases-of-gavelkind-and-tanistry-legal-imperialism-in-ireland-16031610/BAE3970F7BC1F265FB7225F0FA1FAC3F) |
| **Inner Asian dynastic polities** | Sovereignty could be associated with a ruling clan collectively, with assemblies confirming a claimant and several branches possessing standing. | Model branch entitlements, assembly participation, and territorial power bases—not just a father–son chain. [Cambridge University Press](https://www.cambridge.org/core/books/abs/cambridge-history-of-inner-asia/migrations-ethnogenesis/78643831A3F8E34B1DD5873E3D6A46B9) |
| **Mamluk Egypt** | Military status, patronage, heredity, and dynastic continuity interacted; “pure nonhereditary military selection” is too simple. | Allow competing principles within the same polity and across successive reigns. [Cambridge University Press](https://www.cambridge.org/core/journals/bulletin-of-the-school-of-oriental-and-african-studies/article/abs/caught-between-heredity-and-merit-the-amir-qusun-and-the-legacy-of-alnasir-muhammad-b-qalawun-d-13411/B59E735943C9DDFC31D06BA1F491DA67) |
| **Imperial China** | Minority rule’s consequences varied with political organization; concentrated executive authority could make an incapable holder more consequential. | Make regency outcomes depend on where decision powers actually reside. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0147596724000453) |
| **Asante/Akan institutions** | Matrilineal royal descent and queen-mother authority are central. Matrilineal succession does not mean that the ruler must be female. | Store lineage membership independently from father–son descent, and represent female political offices with their own powers. [The New York Academy of Sciences](https://nyaspubs.onlinelibrary.wiley.com/doi/10.1111/j.1749-6632.1997.tb48124.x) |
| **Haudenosaunee Confederacy** | Authority is distributed across titled chiefs, clan mothers, and collective bodies. Clan mothers participate in selection and can remove chiefs; confirmation and condolence procedures matter. | Do not create an artificial confederate king. Apply succession to constituent offices and preserve collective decision rules. [Haudenosaunee Confederacy](https://www.haudenosauneeconfederacy.com/government/) |
| **Oromo gada** | Five classes progress through grades, with leadership rotating every eight years. Membership and participation are structured, not equivalent to universal adult suffrage. | Rotation applies to a governing class and its institutions, not merely the oldest available individual. [UNESCO ICH](https://ich.unesco.org/en/RL/gada-system-an-indigenous-democratic-socio-political-system-of-the-oromo-01164) |

These examples should be authored as **compositions of primitives**, not hardcoded civilizations.

### 3.3 Industrial and modern settings

For industrializing and modern scenarios, distinguish a person’s departure from turnover in the organizations supporting government. Formal leadership datasets identify the **effective leader**, which need not be the ceremonial officeholder. Electoral datasets, meanwhile, record election events that do not necessarily produce a new executive. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022343308100719)

TCE should therefore permit combinations such as a hereditary ceremonial office with an elected executive, a party-selected executive with no competitive public election, or an elected ruler who retains office because the coercive apparatus rejects defeat.

Modernity does not eliminate institutional hybrids: Malaysia’s documented procedure remains a particularly clear example of nomination order, restricted election, and fixed term operating together. [Majlis Raja-Raja](https://www.majlisraja-raja.gov.my/en/election-of-his-majesty-yang-dipertuan-agong)

For constitutional elections, the stabilizing mechanism is not a ballot animation. It is credible compliance with procedures, enforceable limits on winners, and tolerable prospects for losers. [OUP Academic](https://academic.oup.com/qje/article-abstract/126/4/1661/1923169)

---

## 4. Stylized facts and validation targets

A correct simulation should reproduce **conditional patterns**, not force every world to reach the same succession system.

### 4.1 Succession concentrates risk, but most transitions need not become wars

The European death-shock estimates imply a substantial increase in civil-war onset, while still leaving nonwar outcomes much more common in the specified succession years. **TCE test:** crises should cluster around vulnerable transfers without making every death a rebellion trigger. [Göteborgs universitet](https://www.gu.se/sites/default/files/2020-05/2017_9_Kokkonen_Sundell.pdf)

### 4.2 Ruler security, orderly succession, and regime survival can diverge

The modern comparative evidence allows an incumbent to depart peacefully but an accession to fail; it also allows a regime to survive a dictator’s death without establishing that the transfer was peaceful. **TCE test:** the three corresponding statistics must be capable of moving independently. [Anne Meng](https://www.annemeng.com/uploads/5/6/6/6/56666335/jcr_got.pdf)

### 4.3 Genealogical clarity does not imply superior selection for every task

Smith’s Eurasian study argues for a tradeoff between succession arrangements favoring continuity and military-political environments favoring selection among capable competitors. Treat this as a supported comparative argument, not a settled universal law. **TCE test:** a stable dynasty can still lose to a less internally stable but more militarily effective coalition. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/storm-from-the-steppes-warfare-and-succession-institutions-in-premodern-eurasia-10001799-ce/5C02DCBEA731533340E3D6B144C32924)

### 4.4 More relatives must not mechanically produce more rebellion

The European family-size findings contradict that simple relationship. **TCE test:** adding relatives with shared interests and useful connections should sometimes improve stability, while adding independently backed rivals can worsen it. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/715065)

### 4.5 Minority should produce institutional dependence, not predetermined collapse

Chinese evidence makes minority an important stress condition, but its effects vary with institutional context. **TCE test:** the same child ruler can preside over continuity under an accepted regency or a crisis where powers and guardianship are contested. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0147596724000453)

### 4.6 Concessions during succession need not accumulate permanently

Research on succession and representative institutions in 16 medieval European polities distinguishes temporary concessions from lasting power-sharing. **TCE test:** a claimant can promise privileges, later face incentives to revoke them, and succeed or fail according to the beneficiaries’ enforcement capacity. [DOI](https://doi.org/10.1111/1475-6765.12381)

### 4.7 Peaceful removal can be evidence of stability

The Haudenosaunee example includes authorized removal of an unsatisfactory chief. **TCE test:** successful enforcement of an institution’s removal procedure should not automatically lower a generic stability score. [Onondaga Nation](https://www.onondaganation.org/government/clan-mothers/)

Evaluate these patterns across many seeded worlds and matched institutional scenarios. One long-lived dynasty or one spectacular succession war is not an adequate validation sample.

---

## 5. Recommended individual-agent implementation

### 5.1 Minimum persistent state

| Entity | Essential fields |
| --- | --- |
| **Office** | Holder; jurisdiction; powers; term; succession rule; vacancy state; delegated powers |
| **Succession rule** | Eligibility predicates; ranking procedure; selectors; nomination timing; confirmation requirements; contingencies |
| **Person** | Age; acknowledged parentage; lineage memberships; offices; skills; affiliations; resources; beliefs; relationships |
| **Claim** | Candidate; office; legal or customary basis; endorsers; disputed facts; creation date |
| **Political organization** | Membership; leadership; assets; decision procedure; internal cohesion |
| **Coalition** | Participating people and organizations; commitments; claimant supported; resources actually available |
| **Settlement agreement** | Promised offices, property, protection, autonomy, or future turns; guarantors; breach conditions |
| **Transition episode** | Trigger; candidates; decisions; recognition history; coercive actions; outcome flags and dates |

Maintain **acknowledged legal genealogy separately from biological parentage where necessary**. Agents need not know the simulation’s true parentage data. Adoption, disputed legitimacy, and recognition can therefore matter through the society’s rules and available evidence.

### 5.2 Legitimacy should be an actor-specific judgment

For each politically relevant actor \(i\) and claimant \(c\), maintain a small vector rather than one universal legitimacy score:

\[
L\_i(c)=
\bigl(
\text{procedural validity},
\text{lineage entitlement},
\text{ritual acceptance},
\text{fitness},
\text{recognized precedent}
\bigr).
\]

Different actors weight these components differently. A council may privilege procedure; a lineage elder ancestry; a commander military fitness. These differences need not always be cynical: sincerely held institutions can conflict.

A separate support calculation incorporates material and security interests:

\[
U\_i(c)=
E[\text{security}+\text{benefits}+\text{policy satisfaction}]
+\text{normative commitment}\_i(c)
-\text{switching and participation costs}.
\]

This is a **proposed modeling form**, not an estimated historical utility function.

### 5.3 Bargaining and escalation

A claimant should consider resistance when:

\[
p\_{\mathrm{win}}V\_{\mathrm{win}}
-(1-p\_{\mathrm{win}})L\_{\mathrm{defeat}}
-C\_{\mathrm{conflict}}
>
U\_{\mathrm{settlement}}.
\]

Here, victory probability comes from the existing military and coalition model. Settlement utility depends on credible guarantees, not the face value of promises.

An episode can then proceed through:

**activation → information dissemination → claims and recognition → bargaining → procedural resolution, coercion, or fragmentation → consolidation.**

This is not a compulsory linear sequence. Negotiation can continue during war; coalitions can merge; a provisional regency can defer final selection.

Crucially, **do not spawn an army because a claimant has a strong claim**. Claimants must obtain troops, money, transport, and supplies through the same institutions and relationships used elsewhere in TCE.

### 5.4 Separate three maps of power

Keep distinct records of:

**Legal authority:** who is entitled to issue an order.

**Recognition:** who accepts that entitlement.

**Operational control:** who can actually get the order carried out.

A lawful heir might hold the first, a regent the second, and a commander the third. Their overlap determines whether formal succession becomes effective government.

Avoid counting the same soldiers twice because both their commander and their clan have endorsed a claimant. Resource control must resolve to identifiable units, assets, and chains of command.

### 5.5 Make ambiguity substantive rather than random

Useful sources of disagreement include competing rules, not only missing rules:

* Branch seniority versus proximity of kinship.
* Appointment versus hereditary entitlement.
* A prescribed rotation versus a claim of incapacity.
* Different interpretations of which electors or lineages qualify.
* A valid election versus an allegation of coercion or fraud.

For a **synthetic test case**, suppose a ruler dies leaving a designated daughter, a younger brother with a seniority claim, and a council that previously confirmed the daughter. One commander backs the brother; treasury officials recognize the daughter. The outcome should depend on negotiations, control, and accepted interpretation. It should not be decided by randomly selecting which succession enum “wins.”

### 5.6 Performance at 10k–50k people

Most people need not reconsider every claimant every day.

Cache candidate eligibility and genealogical priority, invalidating them on births, deaths, adoption, status changes, and legal amendments. Expand detailed political reasoning around officeholders, selectors, claimants, organizational leaders, and their relevant networks. Other agents participate through already simulated institutions: households, workplaces, assemblies, religious communities, military units, and local authorities.

Use event-driven updates for major developments and slower review during routine politics. Preserve weak or dormant claims without continuously simulating an elaborate conspiracy for each one.

This retains individual agency without requiring an all-to-all political calculation.

### 5.7 Existing models and games worth borrowing from

| Model or game | Useful contribution | What not to import uncritically |
| --- | --- | --- |
| **Egorov and Sonin, *The Killing Game*** | History-dependent expectations about sparing or executing defeated contenders. | Its stylized strategic assumptions are not empirical execution probabilities. [IDEAS/RePEc](https://ideas.repec.org/a/eee/reecon/v69y2015i3p398-411.html) |
| **Fearon, *Self-Enforcing Democracy*** | Public procedures, collective enforcement, armed refusal, and future chances of office. | Do not assume a written electoral procedure enforces itself. [OUP Academic](https://academic.oup.com/qje/article-abstract/126/4/1661/1923169) |
| **Crusader Kings III regency system** | Distinguishes ruler and regent, temporary and entrenched regencies, delegated powers, coalition recruitment, and attempted usurpation. | Its developer explicitly describes balancing historical inspiration against player experience. Borrow the state decomposition, not its calibrated game thresholds. [Paradox Plaza Forum](https://forum.paradoxplaza.com/forum/developer-diary/dev-diary-122-regencies-and-elegance-of-the-empire.1576961/) |
| **Crusader Kings III administrative-government content** | Offers a game-design example of office competition and imperial appointment rather than treating every position as ancestral landholding. | TCE should derive behavior from its own institutional and resource model rather than reproduce a civilization-specific ruleset. [Paradox Interactive](https://www.paradoxinteractive.com/media/press-releases/paradox-interactive/crusader-kings-iii-on-console-gets-major-expansion-and-event-pack) |

**Recommended simplification:** implement one general system for claims, selectors, delegated powers, organizational support, bargaining, and coercion. Supply historically informed templates as data. Avoid separate bespoke engines for “feudal succession,” “tribal succession,” and “modern succession.”

---

## 6. Sources, datasets, and evidence limitations

### Core quantitative and theoretical reading

| Source | Main use |
| --- | --- |
| **Kokkonen & Sundell (2014), “Delivering Stability—Primogeniture and Autocratic Survival in European Monarchies 1000–1800,” *American Political Science Review* 108(2):438–453.** | Comparative deposition and tenure benchmarks; distinguish final publication from earlier working papers. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/delivering-stabilityprimogeniture-and-autocratic-survival-in-european-monarchies-10001800/2399079C174599A840E5230E8827609C) |
| **Kokkonen & Sundell (2020), “Leader Succession and Civil War,” *Comparative Political Studies* 53(3–4):434–468.** | Succession-associated civil-war risk. The numerical estimates marked above come from its 2017 predecessor. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0010414019852712) |
| **Meng (2021), “Winning the Game of Thrones: Leadership Succession in Modern Autocracies,” *Journal of Conflict Resolution* 65(5):950–981.** | Distinguishing succession planning from successful accession. [Anne Meng](https://www.annemeng.com/uploads/5/6/6/6/56666335/jcr_got.pdf) |
| **Kendall-Taylor & Frantz (2016), “When Dictators Die,” *Journal of Democracy* 27(4):159–171.** | Regime survival after death in office. [Journal of Democracy](https://www.journalofdemocracy.org/articles/when-dictators-die/) |
| **Kokkonen, Krishnarajan, Møller & Sundell (2021), “Blood Is Thicker than Water,” *Journal of Politics* 83(4):1246–1259.** | Kin networks and deposition risk. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/715065) |
| **Smith (2026; online 2025), “Storm from the Steppes,” *American Political Science Review* 120(2):546–563.** | Broader Eurasian comparison and the continuity–military-selection tradeoff. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/storm-from-the-steppes-warfare-and-succession-institutions-in-premodern-eurasia-10001799-ce/5C02DCBEA731533340E3D6B144C32924) |
| **Xiong (2024), “Minor Monarchs: The ‘Bad-Emperor’ Problem in Chinese History,” *Journal of Comparative Economics* 52(4):813–824.** | Minority rule and institutional dependence. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0147596724000453) |

For global institutional templates, the scholarly work on Asante queen mothers, Gaelic legal transformation, Inner Asian dynasties, and Mamluk succession should be read alongside the cited Haudenosaunee, Oromo, and Malaysian institutional descriptions. These sources answer different questions: rules, interpretation, and lived institutional practice—not necessarily comparable event frequencies.

### Datasets to connect to TCE’s event log

| Dataset or source family | Relevant fields | Important limitation |
| --- | --- | --- |
| **Archigos** | Effective leaders, entry and exit dates and modes, personal fate after office. The original 2009 article describes 1875–2004 coverage. | Use the codebook for the particular release; it is not a genealogy or a comprehensive record of all claimants. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022343308100719) |
| **NELDA, version 6** | National election events, 1945–2020; useful for constructing electoral comparison samples. | Elections are not equivalent to leadership changes; legislative and executive events require careful matching. [NELDA](https://nelda.co/) |
| **Study-specific monarchy and succession replications** | Rule coding, reigns, transitions, covariates, and outcome definitions. | Preserve publication version and authors’ coding categories; do not merge superficially similar labels automatically. [Studocu](https://www.studocu.com/row/document/univerzita-j-selyeho/historia/09a-kokkonen-and-sundell/10166680) |
| **UCDP** | Organized-conflict and battle-death definitions for modern comparison. | Its commonly used thresholds include 25 battle-related deaths for active conflict and 1,000 for war within a year. These are reporting conventions, not suitable universal escalation triggers for small TCE polities. [Uppsala University](https://www.uu.se/en/press/press-releases/2024/2024-06-03-ucdp-record-number-of-armed-conflicts-in-the-world) |

### What remains contested or thin

**Causal identification.** Societies do not randomly receive succession rules. Coalitions, military organization, religion, conquest, and administrative development affect both institutional choice and stability. A natural-death design helps identify a succession shock more plausibly than it identifies the causal effect of adopting a law.

**Classification.** Father-to-son succession is not necessarily primogeniture. Election may coexist with dynastic restriction. Seniority can rank branches or individuals. Historical descriptions may summarize contested practice as though it were a settled constitution.

**Measurement.** Narrow deposition coding, domestic murder, civil-war death, and foreign overthrow produce different totals. A chronicler’s description of a “usurper” is also a political judgment, not a neutral ground-truth field.

**Coverage.** Long written ruler lists make some monarchies unusually measurable. Poorer documentation of informal leadership or early farming communities does not imply that their transitions were either perfectly peaceful or exceptionally violent.

**Scale.** Modern national civil wars, medieval dynastic conflicts, and disputes inside a small settlement cannot share an unmodified casualty threshold or territorial definition.

The central design consequence is straightforward: **use succession laws to generate claims and procedures; use people and organizations to determine recognition, bargaining, and enforcement; and let crisis outcomes emerge from their interaction.** Validate the resulting frequencies against carefully matched historical samples, while keeping unmeasured rule types explicitly uncertain.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928b6-6d30-83ea-8e2c-319d290145f0)
