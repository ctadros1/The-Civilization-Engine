# Classifying political regimes for The Civilization Engine

## Executive recommendation

**TCE should classify regimes from a bundle of institutional features, not place societies on a single ladder from chiefdom to monarchy to republic to democracy.** The major political-science datasets measure different things: electoral accountability, constraints on executives, or the organizations that control authoritarian rulers. Their categories are complementary, not interchangeable. [Cogitatio Press](https://www.cogitatiopress.com/politicsandgovernance/article/download/1214/714)

For TCE, the useful output is a layered description:

> **Constitutional monarchy — electoral democracy**  
> **Oligarchic republic — military tutelage**  
> **One-party republic — personalist rule**  
> **Chiefdom — hereditary leadership with a binding council**

These are proposed UI compositions, not categories copied from one dataset.

The underlying simulation should store **who selects officials, who can remove them, who commands coercion, who controls revenue, who participates, and which rules actually bind powerful actors**. A classifier then summarizes that state. The label itself should not grant economic, military, or legitimacy bonuses; those consequences should arise from the institutions and behavior it describes.

---

## 1. What the major typologies actually classify

### Comparative overview

| Framework | Classification and observable foundations | Coverage and implications for TCE |
| --- | --- | --- |
| **V-Dem and Regimes of the World, or RoW** | V-Dem measures electoral, liberal, participatory, deliberative, and egalitarian dimensions. RoW derives four categories: closed autocracy, electoral autocracy, electoral democracy, and liberal democracy. Its central distinction is between merely holding elections and providing meaningful accountability through them. | V-Dem **version 16 was released in March 2026**; its RoW variable covers **1900–2025**. Best source for a multidimensional dashboard and an electoral-democracy classifier—not a universal taxonomy of pre-state societies. [V-Dem](https://v-dem.net/data/the-v-dem-dataset/) |
| **Polity** | Combines measures of executive recruitment, executive constraints, and political participation into an authority score. Its six components distinguish how leadership is recruited and how institutionalized competition and checks operate. | Polity5’s published coverage is **1800–2018**, with a population inclusion criterion of **500,000 or more in the reference year**. Useful for modeling executive authority, but its aggregate score is not a complete measure of inclusion or individual rights. [Systemic Peace](https://www.systemicpeace.org/polityproject.html) |
| **Geddes–Wright–Frantz, or GWF** | Distinguishes authoritarian regimes by their controlling institutions and leadership arrangements: especially **party-based, military, personalist, and monarchical**, including combinations. It separates a leader leaving office from the underlying regime ending. | The original study covers **280 autocratic regimes, 1946–2010**. Particularly useful for identifying the actual governing coalition beneath constitutional titles. These categories are nominal organizational types, not steps on a democracy scale. [Cambridge University Press](https://www.cambridge.org/core/journals/perspectives-on-politics/article/autocratic-breakdown-and-regime-transitions-a-new-data-set/EBDB9E5E64CF899AD50B9ACC630B593F) |
| **Boix–Miller–Rosato, or BMR** | A binary democracy–dictatorship measure based on contested elections and participation: popular selection of the executive, directly or indirectly, and a freely and fairly elected legislature, with a male-suffrage requirement. | The original article covered **1800–2007**; the author-distributed **version 4.0 covers 1800–2020**. Useful as a transparent historical electoral benchmark, but its baseline franchise definition is substantially narrower than equal participation by all adults. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0010414012463905) |

### Important differences that the classifier must preserve

**V-Dem is not one democracy score.** Its separate principles allow electoral accountability, liberal rights, deliberation, participation, and political equality to diverge. A TCE polity could therefore have extensive assembly participation but weak protections against arbitrary punishment, or strong legal constraints with a restricted franchise. The first distinction comes from V-Dem; those particular combinations are proposed simulation cases. [V-Dem](https://v-dem.net/data/the-v-dem-dataset/)

**Polity’s “anocracy” is not a synonym for every hybrid regime.** It is the middle of an aggregate authority scale. A middle score does not, by itself, establish that a society is transitioning, collapsing, or conducting competitive but unfair elections. Furthermore, executive constraints can be imposed by aristocrats, a party organization, or a military council rather than by a democratic electorate. [Systemic Peace](https://www.systemicpeace.org/polityproject.html)

**GWF’s organizational categories require actual control.** For TCE, a military regime should require the officer organization to exercise decisive political authority—not merely a ruler who once served in the army. Similarly, distinguish a party that constrains its leader from a party that the leader controls personally. The original GWF classifications are assigned to regime spells; a continuously updating TCE measure of personalization would be an extension, not an exact reproduction. [BPB](https://bpb-us-e1.wpmucdn.com/sites.psu.edu/dist/0/12570/files/2016/05/GWF-Codebook.pdf?bid=12570)

**BMR’s baseline democracy category is historically permissive about exclusion.** Its male-suffrage criterion should not become TCE’s universal definition of popular inclusion. Keep the published historical classification available as one comparison, while separately reporting participation by women, dependent populations, and excluded status groups. [Cogitatio Press](https://www.cogitatiopress.com/politicsandgovernance/article/download/1214/714)

**Implementation consequence:** do not average these classifications or have them “vote” on the correct regime. Store the structural features once, then expose several explicitly named interpretations.

---

## 2. Premodern categories and hybrid regimes

### Proposed operational vocabulary

The following are **TCE definitions informed by the literature**, not claims that scholars agree on universally necessary and sufficient conditions.

| UI term | Structural evidence TCE should require | What should not be sufficient |
| --- | --- | --- |
| **Council community** | Binding collective decision procedures, with leadership subordinate to an assembly, elders, households, or another recognized constituency. | Small population or absence of a palace. |
| **Big-man leadership** | Leadership depends substantially on an individual’s achievements, generosity, relationships, and ability to maintain a following; authority is not automatically transferred with an inherited office. | Simply being the richest person. This follows Sahlins’s ideal-type distinction between achieved leadership and chiefly position. [Cambridge University Press](https://www.cambridge.org/core/journals/comparative-studies-in-society-and-history/article/poor-man-rich-man-bigman-chief-political-types-in-melanesia-and-polynesia/E392602EF37440C9B50A3FDD10AE2A68) |
| **Chiefdom** | A durable chiefly position and ranked authority coordinating obligations beyond an individual household, often through descent and kinship. Record council constraints and administrative development separately. | Agriculture, inequality, or crossing a population threshold. Chiefly power varies substantially in its economic, military, and ideological foundations. [American Academy of Arts and Sciences](https://www.amacad.org/news/chiefs-perspective-prehistory-modern-failing-states) |
| **Monarchy** | An enduring crown-like office with recognized succession rules. Record whether succession is hereditary, elective, designated, or contested, and whether the monarch actually governs. | A ruler calling themselves king, or a dictator promoting a relative. |
| **Republic** | Public governing offices organized without a crown; office access and succession follow civic procedures such as council selection, election, rotation, or lot. | Democracy. A republic can exclude most inhabitants from power. |
| **Oligarchy** | A restricted, privileged group has durable, decisive control over appointment, removal, or major policy. | Wealth inequality alone. Here the term describes political control; economic concentration is a separate measurement. |
| **Clerical rule / theocracy** | Religious officeholders possess binding governing authority, selection powers, or vetoes. Separately tag constitutional supremacy of religious law. | High religiosity, many temples, or sacred royal imagery. “Constitutional theocracy” can be broader than direct priestly government, so the UI should distinguish these meanings. [Politics at U of T](https://www.politics.utoronto.ca/research-publications/faculty-publications/constitutional-theocracy) |

A chiefdom is partly a description of **political organization**, monarchy describes **headship and succession**, and oligarchy describes **the distribution of control**. Forcing them into one exclusive enumeration loses information.

### Three different meanings of “hybrid”

**Electoral-authoritarian hybridity** concerns the relationship between elections and accountability. In Levitsky and Way’s competitive-authoritarian category, opposition is meaningful but incumbents systematically distort the playing field. This differs from elections in which opposition has no realistic route to power. Neither situation can be diagnosed solely from the incumbent’s vote share. [Journal of Democracy](https://www.journalofdemocracy.org/articles/elections-without-democracy-the-rise-of-competitive-authoritarianism/)

**Organizational hybridity** concerns shared or overlapping authoritarian control: a military-party arrangement, for example, or a personalist ruler operating through a party. This is the dimension emphasized by GWF. [Cambridge University Press](https://www.cambridge.org/core/journals/perspectives-on-politics/article/autocratic-breakdown-and-regime-transitions-a-new-data-set/EBDB9E5E64CF899AD50B9ACC630B593F)

**Measurement ambiguity** means evidence or estimated scores lie near a classification boundary. It does not necessarily mean that the institutions themselves are an unusual mixture. RoW explicitly represents this uncertainty. [Cogitatio Press](https://www.cogitatiopress.com/politicsandgovernance/article/download/1214/714)

TCE should store these separately as, for example, `competition_type`, `controlling_organizations`, and `classification_uncertainty`.

A further distinction is essential: **one-party monopoly is not the same as party-organizational control**. Under the proposed classifier, the former means governing eligibility is monopolized by one party; the latter means the party’s institutions can constrain individual leaders. A personalist ruler can dominate a one-party state.

---

## 3. Mechanisms: how institutional features arise and change

Classification rules describe a regime; they do not explain its emergence. The following mechanisms translate research into simulation rules. Their directions have scholarly support, but the implementation details are proposals rather than universally estimated causal laws.

| Mechanism | Implementable rule | Observable institutional consequence |
| --- | --- | --- |
| **Control of politically useful resources** | Agents controlling appropriable land, trade bottlenecks, stored goods, or labor can finance retainers, administrators, warriors, and ritual specialists. Require actual control and successful mobilization, not surplus alone. | Durable leadership, dependent followers, concentrated command and revenue. Earle emphasizes this connection between political-economic control and power strategies. [American Academy of Arts and Sciences](https://www.amacad.org/news/chiefs-perspective-prehistory-modern-failing-states) |
| **Collective resistance to domination** | Followers can coordinate refusal, replacement, withdrawal of support, or exit. Leadership remains constrained when these actions are feasible and credible. | Strong influence without unrestricted command; collective leadership can persist despite ambitious individuals. Boehm treats egalitarian arrangements as actively maintained, rather than simply lacking leaders. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/204166) |
| **Fiscal bargaining** | When rulers depend on contributions from numerous actors who can coordinate or withhold support, bargaining may exchange revenue for consultation, public goods, and limits on rulers. Dependence alone is insufficient without bargaining capacity. | Broader accountability and institutionalized bargains. Cross-cultural premodern research supports associations between revenue dependence and collective governance. [Sociological Studies](https://www.sociostudies.org/journal/articles/140587/) |
| **Delegation and administrative control** | Officials receive powers, compensation, and reporting obligations. Independent complaints and monitoring expose abuse; inherited revenue assignments can make officials harder to control. | Variation in bureaucratization, principal–agent problems, and local autonomy—not automatically variation in democracy. [Sociological Studies](https://www.sociostudies.org/journal/articles/140587/) |
| **Organizational versus personal control** | A party, dynasty, officer corps, or council retains power when it can coordinate appointments and enforce leadership decisions. Personalization increases when a leader captures these mechanisms and replaces organizational loyalty with personal dependence. | Changing military, party, dynastic, and personalist control profiles. The specific update rule is a TCE operationalization of GWF’s organizational distinctions. [Cambridge University Press](https://www.cambridge.org/core/journals/perspectives-on-politics/article/autocratic-breakdown-and-regime-transitions-a-new-data-set/EBDB9E5E64CF899AD50B9ACC630B593F) |
| **Succession and regime continuity** | On a leader’s exit, resolve succession through existing eligibility and selection rules. Start a new regime episode only when foundational rules or the controlling group change substantially. | A new monarch, general, or party leader need not constitute regime change. [BPB](https://bpb-us-e1.wpmucdn.com/sites.psu.edu/dist/0/12570/files/2016/05/GWF-Codebook.pdf?bid=12570) |
| **Erosion of effective competition** | Allow incumbents to restrict organization, information, candidacy, counting, adjudication, or transfer of office. Reclassify from the resulting opportunities and constraints, even if the constitution is unchanged. | A republic with elections can become electorally authoritarian without abolishing either its name or its legislature. [Journal of Democracy](https://www.journalofdemocracy.org/articles/elections-without-democracy-the-rise-of-competitive-authoritarianism/) |

For TCE, these mechanisms should act on concrete relationships. An attempted purge, for example, should succeed only if enough relevant agents comply. It should not directly add points to a “personalism meter” that subsequently manufactures loyalty.

Likewise, a technological change should alter resources, communication, organization costs, and enforcement opportunities. It should not directly unlock a regime type.

---

## 4. Quantitative parameters: published rules versus engineering defaults

### A. Published coding values

**Confidence below concerns faithful reproduction of the source’s coding rule—not proof that its threshold is a natural historical boundary.**

| Quantity | Published value or range | Units | Source and confidence |
| --- | --- | --- | --- |
| **Polity aggregate** | `DEMOC − AUTOC`; range **−10 to +10** | Index points | Polity documentation; **high** for coding. [Systemic Peace](https://www.systemicpeace.org/polityproject.html) |
| **Suggested Polity categories** | Autocracy **−10…−6**; anocracy **−5…+5**; democracy **+6…+10** | Index-point intervals | Suggested project convention; **high** for documentation, not uniquely authoritative cutoffs. [Systemic Peace](https://www.systemicpeace.org/polityproject.html) |
| **Polity executive constraints, XCONST** | **1–7**; principal anchors include unlimited authority, slight/moderate limits, substantial limits, and executive parity/subordination | Ordered categories | Polity5 manual; **high**. Not a cardinal measure of “percent constrained.” [Systemic Peace](https://www.systemicpeace.org/inscr/p5manualv2018.pdf) |
| **BMR baseline suffrage criterion** | A majority of the male population has voting rights—the approximately **50% male-suffrage boundary** | Population share | BMR criterion as documented in the comparative RoW study; **high** for the distinction, but use the exact selected BMR codebook for implementation details. [Cogitatio Press](https://www.cogitatiopress.com/politicsandgovernance/article/download/1214/714) |
| **GWF electoral inclusion screen** | Eligible voters comprise at least **10% of total population**; its indirect-selection route specifies a selecting body at least **60% directly elected** | Population share; body-seat share | GWF codebook; **high**. These are dataset-specific screens, not recommended modern inclusion standards. [BPB](https://bpb-us-e1.wpmucdn.com/sites.psu.edu/dist/0/12570/files/2016/05/GWF-Codebook.pdf?bid=12570) |
| **RoW electoral-democracy gates** | Multiparty-election and free/fair-election indicators each **>2**; electoral democracy index **>0.5** | Two original-scale estimates; one 0–1 index | V-Dem v16 codebook; **high**. All conditions are required. [V-Dem](https://v-dem.net/documents/70/codebook_v16.pdf) |
| **Additional RoW liberal-democracy gates** | Liberal **component** index **>0.8**; transparent enforcement and access-to-justice indicators for men and women each **>3** | 0–1 component; original-scale estimates | V-Dem v16 codebook; **high**. The component is `v2x_liberal`, not the overall liberal-democracy index. [V-Dem](https://v-dem.net/documents/70/codebook_v16.pdf) |
| **Premodern collective-governance measures** | Public goods **10–30**; bureaucratization **5–15**; control over rulers **6–18** | Summed coding scores | Blanton and Fargher’s comparative study; **high** for their instrument, **moderate** for broader comparability. These are not democracy cutoffs. [Sociological Studies](https://www.sociostudies.org/journal/articles/140587/) |

Two implementation cautions follow.

First, **V-Dem’s indicator estimates are not raw simulation event percentages**. Porting its thresholds requires a documented mapping from simulated evidence to indicator meanings. An internal TCE score of `0.6` cannot simply be substituted for `v2x_polyarchy`. [V-Dem](https://v-dem.net/documents/70/codebook_v16.pdf)

Second, retain nonordinary states explicitly. Polity’s special values **−66, −77, and −88** denote interruption, interregnum, and transition; they should not be processed as ordinary negative scores. In TCE, these are better represented by typed states than numeric sentinels. [Systemic Peace](https://www.systemicpeace.org/inscr/p5manualv2018.pdf)

### B. Proposed TCE tuning values

These are **uncalibrated engineering starting points proposed here**. Their ranges are for sensitivity testing, not historical estimates.

| Parameter | Starting value and test range | Units | Purpose and confidence |
| --- | --- | --- | --- |
| Routine classifier refresh | Every **30 days**; test **7–90** | Simulation days | Avoid per-person, per-tick classification. Engineering confidence only; benchmark in TCE. |
| Behavioral evidence window | **24 months**; test **12–36** | Simulation months | Summarize gradual changes. Preserve election evidence across the relevant term rather than expiring it mechanically. |
| “Broad franchise” display threshold | **90%** of the defined adult governed population; test **80–95%** | Adult-population share | A UI convention, not a scholarly democracy boundary. Always display systematic group exclusions separately. |
| “Dominant controller” display tag | Approximately **⅔** of a defined control-domain measure; test **0.60–0.80** | Normalized domain score | Only a summary tag. A binding veto or removal power must not disappear because its holder has a low aggregate score. |
| Gradual-change label persistence | **6 months**; test **3–12** | Simulation months | Reduce oscillation near noisy boundaries. Do not delay explicit coups, abolished offices, or newly effective constitutional rules. |
| Chiefdom/state population gate | **None** | Persons | Classify institutional structure; keep population as explanatory context rather than a universal threshold. |

The distinction between **classification thresholds**, **measurement windows**, and **behavioral parameters** matters. A six-month display rule is not evidence that historical regime transitions normally take six months.

---

## 5. Variation across eras and regions

“Era” should describe circumstances, not prescribe government. Political arrangements must remain available whenever agents can sustain their constituent institutions.

| Setting | Evidence and variation | Consequence for TCE |
| --- | --- | --- |
| **Foragers** | Boehm emphasizes collective resistance to domination. Arnold’s research on complex hunter-gatherers connects chiefly politics and ascribed power to elite control of labor. Foraging therefore does not imply one uniform political arrangement. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/204166) | Permit both constrained leadership and durable hierarchy before agriculture. Track household autonomy, labor control, and collective resistance. |
| **Early farming and horticultural societies** | Sahlins’s Melanesian/Polynesian comparison distinguishes achieved big-man leadership from chiefly position. These are useful ideal types, not rules assigning one regime to every society in either region. [Cambridge University Press](https://www.cambridge.org/core/journals/comparative-studies-in-society-and-history/article/poor-man-rich-man-bigman-chief-political-types-in-melanesia-and-polynesia/E392602EF37440C9B50A3FDD10AE2A68) | Separate personal following from institutional office, and measure how authority survives the incumbent. |
| **Preindustrial regional polities** | Earle’s comparisons show different combinations of land control, warfare, and trade in Hawai‘i, the Andes, and southern Scandinavia. His Andean examples include constrained wartime chiefs, contrasting with more concentrated power elsewhere. [American Academy of Arts and Sciences](https://www.amacad.org/news/chiefs-perspective-prehistory-modern-failing-states) | The same broad “chiefdom” label should accommodate markedly different constraints, inequality, and administrative reach. |
| **Preindustrial states and cities** | Collective-governance research finds variation across regions, including Ming China, Asante and Lozi, Mughal India, and Mediterranean cases. Archaeologists separately argue that Tlaxcallan in Mesoamerica had republican organization. Collective institutions are not exclusively European, and “republican” does not establish modern universal democracy. [Sociological Studies](https://www.sociostudies.org/journal/articles/140587/) | Allow monarchies with meaningful accountability, councils within large states, and non-European republic-like arrangements. Do not infer mass suffrage from public architecture alone. |
| **Industrializing societies** | Aguilar Rivera and Posada-Carbó document early universal male suffrage in parts of Spanish America, particularly New Granada, with comparisons to Argentina and Mexico. Some adopted it in the 1850s, when it remained unusual elsewhere in the West. [OUP Academic](https://academic.oup.com/past/article-abstract/256/1/165/6460290) | Do not require a European industrial-development sequence before franchise expansion. Legal enfranchisement and effective participation need separate measures. |
| **Modern societies** | GWF and RoW classify organizational control and electoral accountability separately; Hirschl examines combinations of constitutional government and religious authority. These distinctions cut across regions. [Cambridge University Press](https://www.cambridge.org/core/journals/perspectives-on-politics/article/autocratic-breakdown-and-regime-transitions-a-new-data-set/EBDB9E5E64CF899AD50B9ACC630B593F) | Keep headship, religious-law supremacy, electoral accountability, and actual controlling organizations independent. |

The evidence also imposes a scale warning. TCE’s **10k–50k-person worlds are below Polity’s stated country inclusion threshold**, and often far below the scale of the modern states used in these datasets. Institutional logic can be adapted; modern country-level frequencies and transition rates should not be transplanted without qualification. [Systemic Peace](https://www.systemicpeace.org/polityproject.html)

For ancient cases, institutional reconstruction is frequently less certain than modern coding. A large temple establishes religious investment more readily than it establishes priestly veto power. TCE can know its own institutional state exactly; it should not imitate the uncertainty of archaeological evidence unless that uncertainty is part of the player’s information model.

---

## 6. Designing the structural classifier

### 6.1 Represent rules, offices, and relationships first

A useful scholarly precedent is **Institutional Grammar**, which decomposes institutional statements and allows multiple rules to be assembled into a jurisdiction-level description. DeMattee’s work specifically addresses overlapping and sometimes contradictory legal rules. This fits TCE’s authored-building-block approach better than a government-type menu. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/psj.12488)

A proposed minimal structure is:

```
Institution
    jurisdiction
    offices[]
    membership_and_eligibility_rules[]
    selection_and_removal_rules[]
    decision_rules[]
    revenue_rights[]
    enforcement_resources[]
    appeal_and_veto_rules[]

Office
    incumbent
    selector_body
    eligible_candidates
    succession_rule
    tenure_rule
    powers
    removal_authorities
    appointment_authorities

Relationship
    actor_or_body
    target
    kind: selects | removes | commands | funds | vetoes | appoints
    legal_basis
    practical_support
    evidence
```

Represent customs in the same framework where possible. A succession convention need not be written to constrain behavior.

Individuals need persistent membership and influence-relevant attributes: household and kin ties, office, organizational membership, assets, command relationships, loyalties, reputation, and remembered commitments. Ordinary citizens need not each possess a comprehensive political strategy, but their participation, compliance, and association decisions must affect the bodies claiming to represent them.

### 6.2 Keep de jure and de facto state separate

**De jure** is the rule set: who is legally eligible, who supposedly appoints judges, whether ministers require council consent.

**De facto** is the effective relationship: whether excluded candidates can organize safely, whether the ruler obeys adverse decisions, whether soldiers follow the cabinet or a general.

For each important feature, retain something like:

```
formal_rule
effective_status
supporting_events
contradicting_events
evidence_count
last_updated
```

Do not define actual power solely by observed victories. An institution may be powerful precisely because others anticipate its veto and never submit unacceptable proposals. The proposed measurement therefore combines the authority graph, credible organizational support, and realized behavior.

### 6.3 Measure participation without hiding exclusion

For TCE’s inclusive dashboard, define:

\[
S\_{\mathrm{legal}}
=
\frac{\text{adults legally eligible to participate}}
{\text{adult population governed}}
\]

Report the denominator explicitly, including how noncitizens and dependent territories are treated. Do not let a polity improve its inclusion score merely by stripping excluded residents of citizenship.

Also retain group-specific eligibility and effective access:

\[
S\_g
=
\frac{\text{eligible adults in group }g}
{\text{all adults in group }g}.
\]

Keep **eligibility, practical access, and turnout** separate. Voluntarily declining to participate is different from being legally excluded or physically prevented from participating.

For a restricted civic assembly, display both perspectives: “participation among recognized citizens” and “participation among governed adults.” This proposed dual reporting is particularly important for historically inspired societies.

### 6.4 Evaluate real routes to governing power

Follow selection chains to the offices that actually govern:

> voters → assembly → ministers → administration and coercive command

A ceremonial head of state should not override that chain. Conversely, an elected assembly should not establish accountability when an unelected actor can cancel its choices.

For electoral accountability, require a conjunction of conditions: a meaningful choice, the ability to organize and communicate, credible selection procedures, access to effective governing office, and the practical possibility of replacing incumbents. TCE should evaluate these features directly rather than requiring a historical alternation before recognizing accountability.

For **direct assemblies, consensus procedures, and selection by lot**, provide functional classifications rather than assuming that an absence of modern parties means closed autocracy. The relevant modern dataset adapter may be inapplicable; the TCE institutional description remains valid.

### 6.5 Treat constraints as a domain-specific structure

Do not collapse control of appointments, taxation, command, justice, and legislation into one mandatory “power share.”

A council might control taxation while a monarch controls appointments. A priesthood might have a narrow but binding veto over religious law. An officer corps might determine leadership succession without managing routine administration.

For observed compliance with binding constraints, a simple diagnostic is:

\[
C\_{\mathrm{observed}}
=
\frac{\text{binding adverse decisions obeyed}}
{\text{binding adverse decisions encountered}}.
\]

Always accompany it with the number and importance of cases. When the denominator is zero, the empirical measure is **unobserved**, not zero and not perfect compliance.

### 6.6 Separate regime, government, leader, and sovereignty

Maintain distinct identifiers for:

* the continuing polity;
* the regime episode;
* the current government or governing coalition;
* individual officeholders.

These identifiers answer different questions. A government reshuffle need not end the regime; a regime transformation need not dissolve the polity.

Also classify jurisdictions separately. A self-governing town, tributary district, and royal core can have different institutions within one larger political order. Occupation, disputed sovereignty, and fragmented territorial control should be separate states or modifiers—not automatically “autocracy.”

### 6.7 Classification pipeline

The following is **architectural pseudocode**, not an exact implementation of any published dataset:

```
snapshot = summarize_institutions_and_behavior(polity, time)

organization = classify_political_organization(snapshot)
headship     = classify_headship_and_succession(snapshot)
controllers  = identify_effective_controlling_bodies(snapshot)
participation = evaluate_inclusion_and_access(snapshot)
competition   = evaluate_routes_to_replacement(snapshot)
constraints   = evaluate_binding_limits_by_domain(snapshot)
religion      = evaluate_religious_governing_authority(snapshot)
sovereignty   = evaluate_jurisdiction_and_control(snapshot)

description = compose_labels(
    organization, headship, controllers,
    participation, competition, constraints,
    religion, sovereignty
)

comparisons = run_versioned_dataset_adapters_where_applicable(snapshot)

return {
    description,
    feature_breakdown,
    comparisons,
    evidence_and_uncertainty
}
```

The compact UI should show one principal description and one or two important modifiers. An expanded panel should explain the decisive features:

> **Oligarchic republic**  
> The governing council selects the executive. Council membership is restricted to recognized merchant households. Most adult inhabitants cannot enter the council or choose its members.

That is more useful than an unexplained “oligarchy: 73%” result.

### 6.8 Performance and simplification

For 10k–50k persistent agents, I recommend **event-driven institutional updates plus periodic classification**. Maintain counters and bounded evidence summaries when appointments, elections, disobedience, arrests, vetoes, and organizational membership change.

Avoid repeatedly searching all person-to-person relationships. Use indexed memberships, explicit command structures, and bounded personal networks. Routine political calculations can operate at the household, council, faction, and institution level while individuals remain persistent and continue daily life.

Start with explicit rules and explainable decision trees. A learned classifier may later help compare outputs with human judgments, but it should not become the authority on whether an institution exists.

These are engineering recommendations, not measured performance claims.

### Existing models to borrow from

| Precedent | What to reuse | What it does not solve |
| --- | --- | --- |
| **RoW aggregation rules** | Transparent classification from multiple observable dimensions, plus boundary uncertainty. | Premodern institutional emergence or TCE’s full label vocabulary. [Cogitatio Press](https://www.cogitatiopress.com/politicsandgovernance/article/download/1214/714) |
| **Institutional Grammar** | A compositional representation of rules and their jurisdiction-level combinations. | Actual compliance and informal control must still be simulated. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/psj.12488) |
| **Epstein’s civil-violence ABM** | An example of generating collective behavior from heterogeneous agents and local interactions. | Epstein explicitly notes that the model does not represent a political order or its replacement. It is not a regime classifier or constitution-making model. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC128592/?utm_source=chatgpt.com) |

The reusable pieces exist, but they solve different parts of TCE’s problem.

---

## 7. Stylized facts, validation, and evidence limits

### Empirical patterns worth reproducing

**Regime classifications should disagree sometimes.** The original RoW comparison reported disagreements with other classifications in approximately **7–12% of country-years**. A TCE system that always makes its Polity-like, BMR-like, and RoW-like outputs identical is probably erasing real conceptual differences. [Cogitatio Press](https://www.cogitatiopress.com/politicsandgovernance/article/download/1214/714)

**Autocratic regime breakdown is not equivalent to democratization.** GWF report that approximately **half of autocratic regime changes lead to another autocracy**. Use this as a conditional modern comparison—not a universal probability attached to every rebellion or ruler death. [Cambridge University Press](https://www.cambridge.org/core/journals/perspectives-on-politics/article/autocratic-breakdown-and-regime-transitions-a-new-data-set/EBDB9E5E64CF899AD50B9ACC630B593F)

**Electoral institutions and authoritarianism frequently coexist.** In V-Dem’s end-2025 classification of **179 countries**, there were **35 closed autocracies, 57 electoral autocracies, 56 electoral democracies, and 31 liberal democracies**. These counts are a dated benchmark for comparable modern scenarios, not a target distribution for every generated world. [V-Dem](https://www.v-dem.net/documents/75/V-Dem_Institute_Democracy_Report_2026_lowres.pdf)

**Boundary uncertainty is substantial.** The same report identifies **13 electoral democracies near the lower democratic boundary and eight electoral autocracies near the upper boundary**. Alternative treatment of those cases yields **74–95 democracies**, compared with the principal classification of 87. This is a classification-ambiguity range, not an ordinary confidence interval for a sampled population count. [V-Dem](https://www.v-dem.net/documents/75/V-Dem_Institute_Democracy_Report_2026_lowres.pdf)

**Premodern governance varies within and between regions.** Blanton and Fargher’s comparison used **30 states, including 10 in sub-Saharan Africa**. It demonstrates comparative variation, but its evidence-rich, selected sample should not be used to estimate the worldwide historical frequency of collective government. [Sociological Studies](https://www.sociostudies.org/journal/articles/140587/)

### Essential synthetic tests

These are proposed tests of classifier logic, not historical claims.

| Test polity or intervention | Expected result |
| --- | --- |
| A ceremonial monarch coexists with freely chosen ministers and enforceable checks. | Monarchical headship can coexist with electoral or liberal democracy. |
| A council of privileged households selects public officials while excluding most adults. | Oligarchic republic, not broad democracy. |
| One party repeatedly wins genuinely competitive elections. | Do not infer one-party monopoly from repeated victories. |
| A ruler personally controls appointments and coercion through a nominal party. | Personalist control; party-monopoly status reported separately. |
| Religious observance is widespread, but religious offices have no binding governing powers. | Not clerical rule merely because society is religious. |
| An officer council replaces one general with another under unchanged rules. | Leader change without necessarily ending the regime episode. |
| A partyless assembly permits broad participation and binds its delegates. | A participatory council polity; do not automatically apply the closed-autocracy label. |
| Population doubles while institutions and effective power relations remain unchanged. | No automatic regime-class change. |
| Citizenship is narrowed to exclude a previously disenfranchised population. | No artificial improvement in the governed-population inclusion measure. |

Run these tests before calibrating historical distributions. Then compare scenario classifications against multiple independent human coders, preserving disagreements rather than tuning every case to one assumed answer.

### Contested claims and thin evidence

**Democracy measurement remains disputed.** Little and Meng argue that trends in some expert assessments diverge from more directly observable electoral indicators. Knutsen and colleagues respond that election outcomes omit important institutional deterioration and dispute the inference of systematic expert pessimism. The practical lesson is to retain both event-level evidence and institution-level judgments, rather than treating either as infallible. [Cambridge University Press](https://www.cambridge.org/core/journals/ps-political-science-and-politics/article/measuring-democratic-backsliding/9EE2044CDA598BD815349912E61189D8)

**Historical causal mechanisms are less securely quantified than modern coding rules.** Resource control, collective resistance, and fiscal bargaining supply plausible mechanisms, but they do not establish universal coefficients linking surplus, population, or technology to a regime type. The comparative and archaeological sources support conditional relationships and counterexamples more strongly than universal transition probabilities. [American Academy of Arts and Sciences](https://www.amacad.org/news/chiefs-perspective-prehistory-modern-failing-states)

**Versioning is part of reproducibility.** Freeze dataset releases and classifier rules for each calibration exercise. V-Dem warns against comparing absolute scores across different releases because historical estimates and measurement procedures can change. Store both the institutional snapshot and the classification version so old worlds can be reclassified without rewriting their history. [V-Dem](https://v-dem.net/data/the-v-dem-dataset/)

### Final design decision

For TCE, make **institutions the simulation state, effective power the measured state, and regime names the explanatory output**.

That architecture permits monarchs to become ceremonial, councils to become oligarchic, parties to become personal instruments, and chiefdoms to develop more differentiated administration without any scripted historical sequence. It also makes every label answerable to a concrete question:

**Which people and institutions can actually make, block, enforce, and change the rules?**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556-tce-research/c/6ab928dc-f00c-83ea-9b19-840a96fc3625)
