# Preconditions and the Industrial Revolution: a simulation-ready model for TCE

## Central conclusion

**Industrialization should emerge when people can repeatedly build—or sustainably import—machines that are worth operating, maintain them, train their replacements, and expand the markets and infrastructure that support them.** Discovering a steam engine is neither necessary for the first mechanized factories nor sufficient for an industrial economy. Early factory development used water power, while steam’s large aggregate productivity contribution arrived considerably later. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/objects/co8411373/arkwrights-water-frame?utm_source=chatgpt.com)

For TCE, the most useful distinction is:

**Invention → working prototype → commercially or strategically viable installation → diffusion → a reproducible industrial system.**

Each transition should be separately possible to fail. A society might possess impressive machines but lack affordable fuel; operate profitable textile mills without developing heavy industry; import machinery successfully; or lose a manufacturing capability when its maintenance and training networks collapse.

This is a synthesis for modeling, not a claim that historians have agreed on one explanation. The literature supports several interacting mechanisms, but disputes their relative importance and the interpretation of important measurements.

---

# 1. Mechanisms: what must come together?

## 1.1 Competing explanations and their implementable implications

| Explanation | Historical argument and evidence | Rule for TCE | Important qualification |
| --- | --- | --- | --- |
| **High wages and cheap energy: Robert Allen** | Expensive labor relative to capital and energy made labor-saving machinery unusually attractive in Britain. Subsequent improvements could make the same technologies profitable elsewhere. | Establishments compare **task-specific labor costs** with the capital, fuel, maintenance, and financing costs of available techniques. Entrepreneurs search harder around expensive tasks. | The mechanism is persuasive in principle; its application to early spinning is contested. Humphries and Schneider find poorly paid women and children rather than the strongly rising spinning wages the explanation would suggest. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/j.1468-0289.2010.00532.x) |
| **Credible institutions and finance** | Secure claims and credible government commitments can encourage investment. North and Weingast emphasize post-1688 England; Sussman and Yafeh challenge an immediate financial transformation. | Contract enforcement, confiscation risk, credit access, entry restrictions, and political connections affect expected returns and who can invest. | Do not implement “constitutional government unlocked: cheaper capital.” British borrowing costs remained high and volatile for decades after 1688. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/constitutions-and-commitment-the-evolution-of-institutions-governing-public-choice-in-seventeenthcentury-england/2E0D2B2D3490BE5C556D836ACB096362) |
| **Useful knowledge, science, and culture: Joel Mokyr** | Experimentation, measurement, publication, and communication between practical and learned people made improvements easier to generate and sustain. | People acquire specific skills and methods; institutions affect opportunities to meet, experiment, demonstrate, publish, and recruit. | Represent mutable practices and networks—not a permanent national “scientific culture” bonus. Mokyr’s account is not a calibrated numerical law. [Northwestern Faculty](https://faculty.wcas.northwestern.edu/jmokyr/Berg-replyDec.05.PDF) |
| **Technical human capital** | Evidence from French industrialization associates upper-tail knowledge, proxied by *Encyclopédie* subscriptions, with industrial growth; basic literacy alone explains less. | A small population of capable mechanics, instrument makers, organizers, and teachers can unlock activities unavailable to a much larger population lacking those specialties. | Universal literacy is not a historical prerequisite for initial mechanization. Nor does the French evidence establish a universal minimum number of experts. [National Bureau of Economic Research](https://www.nber.org/papers/w20219) |
| **Energy, land, and geography: Wrigley; Pomeranz** | Coal helped loosen land-dependent constraints on heat and material production. Pomeranz additionally emphasizes coal accessibility and New World resources in explaining divergence. | Price **delivered usable energy**, including transport, conversion losses, and reliability. Let biomass demand compete with other land uses. | Coal underground is not cheap power at a workshop. Water power and imported energy provide alternatives; the timing and extent of Europe–Asia divergence remain disputed. [Cambridge University Press](https://www.cambridge.org/core/books/abs/energy-and-the-english-industrial-revolution/industrial-revolution-and-energy/193609624ECBA7D4E53017D0607C9CFC) |
| **Trade, empire, and coerced production** | Overseas markets and resources were integral to Britain’s trajectory. Research also finds relationships between slave-derived wealth and British industrial development. | Trade supplies inputs, customers, finance, and information. Political power changes access and distributes costs and gains. | Historical contribution does not mean slavery or colonial empire is a universal prerequisite. Estimates of their aggregate causal importance remain debated. [National Bureau of Economic Research](https://www.nber.org/papers/w30451) |
| **Market size and coordinated investment** | Large fixed costs and complementary investments can make individually unattractive projects viable together—the “big push” mechanism. | A mine, transport route, ironworks, and machinery workshop may jointly support one another. Procurement, contracts, or coordinated investment can overcome initial demand gaps. | This is a useful theoretical mechanism, not evidence for a single historical industrialization threshold. [National Bureau of Economic Research](https://www.nber.org/papers/w2708) |

These explanations should mostly **interact**, not compete for one winning national modifier. For example, institutions can finance a canal; the canal changes delivered coal prices; cheaper coal changes machinery profitability; successful machinery raises demand for mechanics and training.

## 1.2 The rules that matter most

### Economic opportunity is local, task-specific, and seasonal

A machine replaces a particular bundle of tasks—not “labor” in the abstract. Compare its costs with the people actually performing those tasks, including their annual availability and alternative employment.

A high male construction wage in a capital city is not automatically the opportunity cost of a rural woman’s seasonal spinning. Likewise, a low daily wage combined with many working days can produce a different household income from a higher wage with intermittent employment. The spinning debate and Japanese work-year evidence make these distinctions essential. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/ehr.12693)

**Implementation:** maintain local wages by occupation and contract, plus household time allocation. Let bargaining, legal restrictions, discrimination, skill scarcity, and seasonal alternatives generate wage differences; do not encode them as inherent productivity differences between demographic groups.

### Specialization requires food and working capital, not a named “Agricultural Revolution”

A full-time mechanic must eat while making something that will only earn revenue later. A growing industrial town must obtain food, whether from its hinterland or trade.

**Implementation:** pay for experimentation, machine construction, and training with actual food, materials, and labor time. Agricultural improvements, food imports, redistribution, and accumulated reserves are alternative ways of supporting specialists. More food can also support more people rather than permanently higher consumption per person—the demographic feedback emphasized by unified growth models. [National Bureau of Economic Research](https://www.nber.org/papers/w6811)

### Machines create new bottlenecks

Accelerating one production stage increases demand for preceding and following stages. Faster spinning is valuable only when usable fiber arrives and yarn can be sold or woven. Cheaper iron increases demand for mining, fuel, transport, and machining.

**Implementation:** production should be a sequence of recipes with inventories, quality requirements, and separate capacities. A textile innovation changes spinning coefficients—not a civilization-wide “textile productivity” scalar. Historical spinning machinery also differed in the strength and fineness of yarn it could produce. [Industrie Museum](https://www.industriemuseum.be/nl/collectie-item/de-mule-jenny-doorgelicht)

### Operating experience matters after invention

A successful prototype need not be reliable, economical, or easy to reproduce. Cartwright’s first power-loom ventures illustrate the gap between patented machinery and a successful business. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/people/ap13666/cartwright-edmund)

**Implementation:** attach performance to machine designs and vintages. Operators learn to reduce breakage; mechanics improve components; builders learn to manufacture more consistently. Keep engineering limits so that experience cannot reduce every input toward zero.

### Knowledge must be carried and transmitted

A drawing can preserve dimensions without preserving all the skills needed to cast, fit, diagnose, and repair a machine. Conversely, practical networks can improve technology without strong exclusive patent rights. Cornish pumping-engine development provides an important case of collective invention and shared performance information. [OUP Academic](https://academic.oup.com/cje/article-abstract/28/3/347/1711845)

**Implementation:** distinguish documented designs, personal know-how, demonstrated local capability, and production experience. Apprenticeships, migration, observation, publication, secrecy, and licensing move different components of knowledge.

### Institutions change incentives and access

Property rights, finance, apprenticeship, procurement, and labor regulation need not arrive as one package.

**Implementation:** institutions alter specific transactions: whether an inventor can borrow, whether a rival can copy, whether an apprentice may open a workshop, whether a worker may leave, and whether inspectors can enforce a law. State-owned, merchant-financed, cooperative, and household enterprises should all be possible, with different objectives and constraints.

---

## 1.3 The technology cluster: a graph, not a ladder

The following dates are **benchmarks in the British development sequence**, not dates at which TCE should unlock anything and not claims about the first appearance of every underlying technique worldwide. The dependency mapping is a proposed engineering abstraction.

| Capability/node | Historical benchmark | Main prerequisites and alternatives | Concrete unlocks |
| --- | --- | --- | --- |
| **Coke-fired iron smelting** | Coalbrookdale, **1709** | Suitable coal and coking; ore, flux, furnace construction, and sustained air blast. Charcoal smelting remains an alternative. | Coke and mineral-fuel pig-iron recipes; expanded foundry supply without proportional charcoal demand. [National Trust](https://www.ironbridge.org.uk/our-story/the-iron-bridge/) |
| **Atmospheric steam pumping** | Newcomen, **1712** | Boiler and cylinder production, pumps, valves, seals, beams, fuel, and water. Does **not** require later precision-machine-tool standards. | Mine-drainage service; deeper workable deposits where drainage is limiting. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/objects/co50900/newcomen-atmospheric-engine) |
| **Multi-spindle hand spinning** | Jenny, **1760s** | Spinning skills, suitable prepared fiber, woodworking, spindles, and transmission components. No steam or factory requirement. | Hand-powered spinning machinery; greater output from suitable spinning tasks. [National Archives](https://www.nationalarchives.gov.uk/education/resources/georgian-britain-age-modernity/spinning-jenny/) |
| **Powered roller spinning** | Water-frame patent, **1769**; surviving museum example c.1775 | Roller mechanisms, prepared fiber, reliable drive power, and organized feeding and maintenance. The jenny need not be a hard prerequisite. | Water-powered spinning mills; stronger yarn suitable for different uses. [Industrie Museum](https://www.industriemuseum.be/nl/collectie-item/de-mule-jenny-doorgelicht) |
| **Improved cylinder boring** | Wilkinson equipment, **c.1775** | Foundry output, cutting tools, rigid supports, measurement, and drive power. | More accurately fitted cylinders and machinery components; improved engine construction and repair. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/objects/co46448/cast-iron-boring-bar-and-boring-head-square-threa-boring) |
| **Separate-condensation steam engine** | Watt patent, **1769** | Steam-pumping knowledge, improved sealing and manufacture, separate condenser, and supporting pumps. | Lower-fuel pumping designs, widening the locations where steam can pay. [DOI](https://doi.org/10.1093/oep%2Fgpab008) |
| **Mule spinning** | Crompton, **1779** | Roller drawing and a moving carriage, suitable machinery construction, and skilled operation. | Finer and stronger yarn; new quality grades, not merely more identical yarn. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/objects/co8405253/spinning-mule) |
| **Rotative steam power** | Watt mechanism, **1781** | An engine plus mechanisms converting reciprocating movement into useful rotary drive. | Steam-driven shaft power for manufacturing; less dependence on suitable millstreams, though water supplies still matter. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/objects/co8404624/haydock-colliery-steam-engine) |
| **Coal-fired iron refining and rolling** | British developments, **1780s** | Pig iron, controlled refining furnaces, skilled handling, and hammering or rolling equipment. | Wrought-iron bars and components with properties distinct from brittle cast iron. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/objects/co19863/two-links-of-an-anchor-chain-forged-from-puddled-iron-with-cast-iron-stubs-late-18th-to-mid-19th-century) |
| **Powered weaving** | Cartwright patent, **1785** | Reliable yarn, coordinated shedding/picking/beating mechanisms, drive power, preparation, and maintenance. Animal, water, or steam power are alternatives. | Mechanized cloth production; initially expensive or unreliable installations are possible. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/people/ap13666/cartwright-edmund) |

Three graph-design consequences follow.

**First, factories should not require steam.** Reuse the existing water-power branch. A factory is an establishment that coordinates workers, machinery, materials, and power—not a synonym for “steam-powered building.”

**Second, avoid circular prerequisites.** Crude engines and existing craft tools can help create better machining capabilities; those capabilities then improve engines. Require minimum tolerances, not fully developed industrial tooling before any industrial tooling can exist.

**Third, distinguish enabling links from improvement links.** Better iron supply makes machinery cheaper; it need not be an absolute prerequisite for every machine. Existing charcoal-based metallurgy can support an initial installation. Likewise, more efficient steam can make an already-known design economical in new locations.

---

# 2. Parameters: evidence, reconstructions, and design priors

Use four confidence labels:

**H:** well documented within its stated scope.  
**M:** historical reconstruction or approximate benchmark.  
**L:** contested estimate or model-dependent reconstruction.  
**P:** proposed TCE calibration prior, **not a measured historical parameter**.

Confidence concerns both the number and its applicability. A well-recorded legal limit is not evidence that everyone obeyed it.

## 2.1 Historical microeconomic and institutional anchors

| Parameter | Value and unit | Appropriate use | Source and confidence |
| --- | --- | --- | --- |
| Early jenny purchase price | Approximately **70 shillings for a 24-spindle machine** | A specific reconstructed capital-cost anchor—not a universal machine price. | Allen, *Industrial Revolution in Miniature*. **M/L**. [Nuffield College](https://www.nuffield.ox.ac.uk/media/2163/allen-spinningjenny.pdf) |
| Jenny labor productivity in Allen’s scenarios | **2–4×** the comparison technique | Sensitivity range for the modeled spinning task; spindle count is not whole-process labor productivity. | Allen. **L**. [Nuffield College](https://www.nuffield.ox.ac.uk/media/2163/allen-spinningjenny.pdf) |
| Domestic spinning utilization in those scenarios | **0.3–0.5 of full-time labor input** | Household time allocation, not factory capacity utilization. | Allen. **L**. [Nuffield College](https://www.nuffield.ox.ac.uk/media/2163/allen-spinningjenny.pdf) |
| Modeled jenny investment return | Central scenario: **Britain 38%; France 2.5%; India −5.2% annually** | Demonstrates sensitivity to relative prices. Uses 3× productivity, 0.4 full-time input, and an assumed 10-year life. These are **not observed firm returns**. | Allen; wage assumptions challenged by Humphries and Schneider. **L**. [Nuffield College](https://www.nuffield.ox.ac.uk/media/2163/allen-spinningjenny.pdf) |
| Fuel saving from improved condensation | **Up to approximately 75% reduction** in the Science Museum’s comparison with earlier atmospheric engines | An upper comparative benchmark for a substantially improved design—not the default gain for every steam installation. | Science Museum Group. **M**. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/objects/co50899/atmospheric-engine-by-john-smeaton-1772-models-atmospheric-engines-newcomen-engines) |
| Japanese work year | Approximately **325 days/year by 1700** in the studied servant-contract evidence | Shows why annual labor supply cannot be inferred from daily wages alone. Not a mean for every Japanese household. | Kumon, *The Labor-Intensive Path*. **M**. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/laborintensive-path-wages-incomes-and-the-work-year-in-japan-16101890/D26C6DD740154EC29102BD21308091F5) |
| Covered teenage factory work, 1833 legislation | **12 hours/day legal ceiling** | A rule governing eligible contracts, not an observation of universal working hours. | UK Parliament. **H** for the rule. [Parliament UK News](https://www.parliament.uk/about/living-heritage/transformingsociety/livinglearning/19thcentury/overview/factoryact/) |
| Initial factory-inspection capacity | **4 inspectors for approximately 4,000 mills** | Enforcement depends on staffing, travel, information, and sanctions—not legislation alone. | UK Parliament. **H** for the reported administrative scale. [Parliament UK News](https://www.parliament.uk/about/living-heritage/transformingsociety/livinglearning/19thcentury/overview/factoryact/) |

The jenny evidence is especially useful as a **model comparison problem**: test Allen-style factor-price incentives against the lower spinning-wage reconstruction. Do not average the disagreement into a supposedly certain parameter.

## 2.2 Macroeconomic calibration anchors

These are reconstructed British trends, not rates to award after researching machinery.

| Period | Real GDP per worker growth | Real consumption-earnings growth |
| --- | --- | --- |
| **1770–1800** | **0.43%/year** | **0.30%/year** |
| **1800–1830** | **0.31%/year** | **0.15%/year** |
| **1830–1860** | **0.92%/year** | **1.01%/year** |

Source: Crafts (2022), using revised historical national accounts and earnings series. **Confidence M**; competing reconstructions should remain available as alternative calibration targets. “Consumption earnings” means earnings adjusted by a cost-of-living index. [DOI](https://doi.org/10.1093/oep%2Fgpab008)

Steam’s estimated contribution to **labor-productivity growth**, in **percentage points per year**, was:

| Period | Steam contribution |
| --- | --- |
| 1760–1800 | **0.01** |
| 1800–1830 | **0.02** |
| 1830–1850 | **0.20** |
| 1850–1870 | **0.41** |
| 1870–1910 | **0.31** |

These growth-accounting estimates support a long diffusion and complementary-investment process, not an immediate economy-wide steam multiplier. **Confidence M**, conditional on the accounting framework. [OUP Academic](https://academic.oup.com/view-large/326681291)

## 2.3 Proposed starting ranges for TCE

The following are deliberately broad **experimental priors**. They should be replaced or narrowed by recipe-specific evidence and calibration.

| Parameter | Initial sensitivity range | Implementation |
| --- | --- | --- |
| Required real return on risky productive capital | **5–20%/year** | Vary by investor, financing arrangement, uncertainty, and political risk. **P** |
| Economic service life of substantial machinery | **8–25 years** | Separate physical wear, repair, and economic obsolescence. **P** |
| Working-capital requirement | **1–6 months of operating expenditure** | Derive actual requirements from wages, inventories, production time, and payment delays. **P** |
| Formation of advanced craft competence | **2–7 years of substantial practice/training** | Applies to difficult construction and maintenance specialties—not every machine operator. **P** |
| Learning improvement | **2–10% reduction in a selected input per doubling of cumulative completed work** | Apply only to improvable coefficients, with lower engineering bounds and possible forgetting. **P** |

Do **not** sample utilization as an arbitrary factory bonus. Derive it from orders, workers, raw materials, available power, breakdowns, and seasonal interruptions.

## 2.4 A worked adoption example

Consider an **illustrative, not historical**, machine:

* Existing technique: **1 labor-hour per unit**.
* Machine technique: **0.4 labor-hours plus 0.1 useful kWh per unit**.
* Annual capital and fixed-maintenance charge: **1,000 account units**.

Its unit-cost saving is:

\[
\text{saving}=0.6w-0.1p\_E-\frac{1000}{Q}
\]

Here \(w\) is account units per labor-hour, \(p\_E\) account units per useful kWh, and \(Q\) annual output.

| Scenario | Wage \(w\) | Energy price \(p\_E\) | Annual output \(Q\) | Saving per unit |
| --- | --- | --- | --- | --- |
| Expensive labor, adequate market | 1.0 | 1.0 | 5,000 | **+0.30** |
| Cheap labor | 0.3 | 1.0 | 5,000 | **−0.12** |
| Small market | 1.0 | 1.0 | 1,000 | **−0.50** |
| Expensive energy | 1.0 | 5.0 | 5,000 | **−0.10** |

One unchanged machine can be profitable in one settlement and unattractive in another. A later improvement can reverse the result without either settlement entering a new era.

---

# 3. Variation across periods and world regions

## 3.1 Use different economic configurations, not compulsory stages

The first two rows below are proposed applications of the same simulation rules, rather than empirically estimated “industrialization equations” for prehistoric societies.

| Configuration | What changes in the model |
| --- | --- |
| **Foraging societies** | Mobility, seasonal resource concentrations, storage, and exchange determine whether fixed facilities and specialist work are worthwhile. Low population alone should not be a prohibition on invention; the relevant constraints are inputs, demand, and supporting labor. |
| **Early farming societies** | Stored food can support longer projects and specialists, but harvest risk and population growth can absorb the gains. Do not turn agricultural surplus automatically into research expenditure. |
| **Pre-industrial commercial societies** | Large markets, sophisticated crafts, extensive trade, water power, and substantial cities can coexist without sustained fossil-fuel industrialization. Commerce and technical competence are not themselves sufficient. |
| **Industrializing societies** | Reproducible machinery, maintenance services, power supply, finance, and transport become mutually reinforcing. Different sectors and districts move at different speeds. |
| **Modern economies** | Formal research, advanced education, electricity, services, and international production networks become more important. Manufacturing employment need not keep rising as manufacturing productivity rises. |

The population–technology feedback is central to Galor and Weil’s framework; Wrigley explains why sophisticated organic economies could still face energy constraints. Modern structural-change models also permit manufacturing employment to peak earlier in development rather than reproducing Britain’s sequence. [National Bureau of Economic Research](https://www.nber.org/papers/w6811)

## 3.2 Regional trajectories and counterexamples

| Region | Historically important variation | Implication for TCE |
| --- | --- | --- |
| **Britain** | Coal costs varied sharply within the country; transport separated cheap coalfield fuel from expensive delivered urban fuel. | Calculate establishment-level energy costs. A national coal-resource flag is inadequate. [Nuffield College](https://www.nuffield.ox.ac.uk/media/2162/allen-industrev-global.pdf) |
| **Continental Europe** | Industrialization involved diffusion, local adaptation, technical expertise, and geographically uneven fuel access. French evidence highlights specialized knowledge; European city evidence associates coal proximity with later growth. | Let imported designs become viable as machinery improves and local complementary capabilities develop. [National Bureau of Economic Research](https://www.nber.org/papers/w20219) |
| **China** | Advanced commercial regions challenge explanations based on a simple absence of markets or ingenuity. Pomeranz and historical national-accounting research disagree over the chronology and degree of divergence. | Compare regions with regions, not a prosperous European core with an entire Asian empire. Do not give large coal reserves automatic economic accessibility. [JSTOR](https://www.jstor.org/stable/j.ctt7sv80) |
| **India** | Highly developed cotton production preceded British mechanized dominance. Relative wages, raw-cotton costs, technology, and transport altered comparative advantage over time. | A sophisticated craft exporter can face mechanized competition without having been technologically ignorant. Model quality, input prices, and market access separately. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/j.1468-0289.2008.00438.x) |
| **Japan** | Labor-intensive development combined extensive household labor use with subsequent industrialization; the British high-wage path was not universal. | Permit techniques that economize on scarce capital and resources while using more labor, followed by selective mechanization. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/laborintensive-path-wages-incomes-and-the-work-year-in-japan-16101890/D26C6DD740154EC29102BD21308091F5) |
| **Egypt** | Muhammad Ali’s early nineteenth-century state pursued textile and other industrial development. | State procurement and investment are possible routes. Model their financing and organizational requirements rather than requiring private entrepreneurship or democracy. [Cambridge University Press](https://www.cambridge.org/core/journals/annales-histoire-sciences-sociales/article/abs/legypte-de-muhammadali-pouvoir-politique-et-developpement-economique/033204E65A4DBACD39AEFCCC97402655) |
| **African regions** | Labor scarcity could coexist with scarce capital, costly transport, ecological constraints, and strongly seasonal labor availability. Colonial economic arrangements also varied substantially. | Scarce labor alone must not guarantee mechanization. Annual averages should not conceal months when household labor has few alternatives. [DOI](https://doi.org/10.4000%2Fpoldev.78) |
| **United States** | Nineteenth-century establishment evidence connects steam adoption, establishment size, and labor productivity. | Power technology and scale should reinforce one another, rather than all workshops receiving the same benefit from steam knowledge. [National Bureau of Economic Research](https://www.nber.org/papers/w11931) |

A particularly important consequence is that **industrialization need not mean convergence on the same production techniques**. Societies can combine imported machinery, abundant labor, water power, state contracts, export agriculture, or specialized manufacturing in different proportions.

---

# 4. Stylized facts and social consequences the simulation should reproduce

## 4.1 Validation patterns

| Pattern | Historical anchor | What a correct simulation should do |
| --- | --- | --- |
| **Gradual aggregate acceleration despite dramatic individual inventions** | The productivity-growth estimates in Section 2 remain modest before 1830. | Sectoral breakthroughs affect aggregate output in proportion to adoption, sector size, and complementary investment—not instantly. |
| **A long lag between early steam and its largest aggregate contribution** | Steam’s estimated contribution rises from about **0.01–0.02 percentage points/year before 1830** to **0.41 in 1850–1870**. | Early engines concentrate in suitable niches; later improvements widen their viable uses. [OUP Academic](https://academic.oup.com/view-large/326681291) |
| **Urbanization over generations** | England and Wales were roughly **one-third urban in 1801**, just over **one-half in 1851**, and just over **three-quarters by 1900**. | Industrial jobs attract migrants, but food supply, housing, transport, and household circumstances constrain movement. Definitions of “urban” must be matched before comparison. [CAMPOP](https://www.campop.geog.cam.ac.uk/blog/2024/08/22/stuck-in-the-mud/) |
| **Large but heterogeneous urban health penalties** | Davenport estimates life expectancy at birth of **32.4 years** across six large non-London English cities in **1838–44**, versus **40.4 nationally**; individual cities differed substantially. | Density and growth interact with water, sanitation, disease exposure, housing, and services. Do not impose one automatic industrial-city mortality penalty. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13109) |
| **Consumer welfare and production can diverge** | Revised evidence still finds slow early consumption-earnings growth, but challenges a large collapse in labor’s national income share. | Track wages, hours, household prices, employment, profits, and rents separately. GDP growth is not a household-welfare score. [DOI](https://doi.org/10.1093/oep%2Fgpab008) |
| **Invention can precede successful commercialization** | Cartwright’s early powered-weaving ventures required repeated development and suffered business failure. | Permit technically working machines, unsuccessful firms, and later successful imitators. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/people/ap13666/cartwright-edmund) |
| **Law and practice differ** | Four initial inspectors faced approximately 4,000 mills under the 1833 factory regime. | Enforcement depends on staffing, information, access, sanctions, and evasion. [Parliament UK News](https://www.parliament.uk/about/living-heritage/transformingsociety/livinglearning/19thcentury/overview/factoryact/) |
| **Innovation can occur through sharing as well as exclusion** | Cornish pumping-engine development involved collective invention. | Patents, secrecy, prizes, procurement, and shared performance reports should produce different innovation and diffusion patterns. [OUP Academic](https://academic.oup.com/cje/article-abstract/28/3/347/1711845) |

The national and large-city observations are **comparison targets**, not direct predictions for a literal 50,000-person world. Their spatial scale matters.

Also, life expectancy at birth of 32 years does **not** mean that most working adults die at 32. Calibrate age-specific mortality; never derive a uniform annual death probability as \(1/\text{life expectancy}\).

## 4.2 Factories change the organization of life

Factories combine technical coordination with authority over when and how work occurs. Thompson’s influential account emphasizes clock discipline and the reshaping of work time, although this should not be read as saying that pre-industrial people lacked schedules or wage discipline. [OUP Academic](https://academic.oup.com/past/article-abstract/38/1/56/1454624)

For TCE, represent this through **employment contracts and daily schedules**. An establishment needs particular roles present together; late arrivals, missing fuel, or a broken drive can interrupt other workers. Owners may prefer longer operating hours to spread fixed costs. Workers weigh wages against fatigue, care obligations, health, travel, and alternatives.

Household effects are essential. A factory wage can increase cash income while reducing time for childcare, food production, home manufacturing, or education. Children’s employment should compete with schooling and care, with consequences persisting into adult skills and health—not function as an inexpensive labor bonus.

**Resistance should have reasons.** Workers threatened with displacement, artisans facing loss of autonomy, or households unable to absorb a transition can oppose machinery even when total output would rise. In the model, compensation, alternative employment, bargaining, and political representation can change those responses.

For coerced labor, never substitute “wage = zero.” Record provisioning, coercive institutions, supervision, resistance, escape, injury, and the losses borne by people and households. An owner’s accounting surplus and social welfare are different quantities.

## 4.3 Do not script an “Engels’ pause”

Allen’s influential interpretation emphasizes delayed wage gains and rising profits. Crafts’s revised accounting finds a smaller productivity–consumption-earnings gap and much less evidence of declining labor share. These are materially different interpretations, not merely different labels. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0014498309000199)

TCE should therefore allow an early industrial welfare lag **without requiring it**. Its severity should emerge from food and housing prices, labor supply, bargaining, working hours, employment transitions, and public services.

Likewise, Davenport disputes a universal dramatic collapse in urban life expectancy during the 1830s–1840s. Industrial growth can worsen health, but the disease environment and urban administration must determine how and where. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13109)

---

# 5. Modeling recommendation for TCE

## 5.1 Minimal representation

| Entity | State worth representing |
| --- | --- |
| **Person** | Occupation-specific skills; practical experience; known designs and methods; social contacts; health; employment; willingness and opportunity to experiment. |
| **Household** | Cash, food and goods; care obligations; housing; debts; members’ time allocation; consumption needs and preferences. |
| **Establishment** | Owner and objectives; inventories; contracts; employees by role; production recipes; machinery; order expectations; finance; maintenance capacity. |
| **Machine/design** | Capability requirements; bill of materials; quality outputs; labor, heat, and shaft-power coefficients; capacity; wear; failure modes; repair requirements; design lineage. |
| **Institution** | Membership; budget; authority; enforcement reach; access rules; lending, training, procurement, publication, or regulatory functions. |
| **Market and transport network** | Delivered prices; accessible suppliers and customers; capacity, travel time, seasonality, interruption risk, and information delays. |

The crucial separation is between **knowing a design**, **being able to construct or import it**, and **being able to use it economically**.

An industrial region need not produce everything locally. Its reproduction can depend on sustainable trade. But imported machinery should require purchases, routes, installation, compatible supplies, and maintenance—not arrive as a permanent free unlock.

## 5.2 Establishment-level adoption

A useful screening equation is:

\[
\widehat c\_{fj}
=
\sum\_m p^{delivered}\_{fm}a\_{jm}
+
\sum\_s w\_{fs}\ell\_{js}
+
p^{useful}\_{fE}e\_j
+
\frac{A(K\_j,r\_f,T\_j)+M\_j}{\widehat Q\_{fj}}
\]

For establishment \(f\) considering technique \(j\):

* \(a\_{jm}\): material inputs per quality-adjusted output unit;
* \(\ell\_{js}\): labor-hours required from each skill or role;
* \(e\_j\): useful-energy requirement;
* \(K\_j\): installed capital cost;
* \(M\_j\): annual fixed maintenance;
* \(\widehat Q\_{fj}\): expected annual saleable output;
* \(A\): annualized capital charge.

For simple screening, \(A=Kr/[1-(1+r)^{-T}]\), or \(K/T\) when \(r=0\).

Keep **heat, mechanical shaft power, and later electricity distinct**. Their conversion and transmission requirements belong in recipes and infrastructure.

This equation is a project-screening approximation, not the accounting system. Actual construction consumes goods and labor; borrowing creates liabilities; depreciation is not another cash payment. Avoid charging both the full purchase price and annualized capital recovery as operating cash expenses.

Agents should compare only a few known alternatives using imperfect forecasts. They can underestimate demand, overestimate reliability, follow a successful neighbor, or remain cautious after a failure.

### Physical feasibility comes first

Before profitability, check:

**required capabilities AND compatible inputs AND a viable power source AND necessary operators/maintainers.**

Alternative supplies and techniques enter through **OR** conditions. A water-powered machine and a steam-driven variant can share an output recipe while differing in capital, siting, and operating requirements.

### Scale comes from orders and bottlenecks

Expected output is bounded by machinery capacity, available workers, materials, power, and expected sales. Do not assume every factory operates at nameplate capacity.

A plant that cannot sell enough output may be unviable despite excellent labor productivity. Conversely, a long-term procurement contract can support scale—but its purchaser must have a budget.

## 5.3 Innovation should be project-based and problem-directed

Create experimentation opportunities around problems people actually encounter: expensive drainage, inconsistent yarn, broken gears, fuel consumption, or a difficult machining operation.

A compact stochastic implementation is:

\[
P(\text{successful trial in }\Delta t)
=
1-\exp[-\lambda\_j E\_p S\_p C\_p\Delta t]
\]

Here \(E\_p\) is effective experimental effort, \(S\_p\) relevant skill adequacy, \(C\_p\) access to complementary knowledge and testing resources, and \(\lambda\_j\) a calibrated difficulty parameter. Define units consistently—for example, years and full-time-equivalent experimenters.

This is a **proposed mechanism**, not an empirically established invention law. Economic conditions influence which projects receive effort. Success yields a design or improvement, not guaranteed profitable adoption.

Use authored candidate capabilities and bounded design variants. TCE does not need unrestricted machine invention to produce divergent histories: varying who combines existing capabilities, where they succeed, and whether others can reproduce them already creates substantial contingency.

## 5.4 Knowledge survival must be tested

A capability survives when enough knowledge and supporting production remain accessible to replace what is lost.

Track whether a workshop can survive the retirement or death of its founder, repair its principal machine, obtain replacement components, and train successors. Documentation reduces some losses but need not replace practical experience.

This creates a stronger definition of industrial maturity than “number of technologies discovered.” Useful dashboard measures include machinery use, sectoral productivity, replacement capacity, dependence on imported specialists, and the depth of local maintenance networks.

## 5.5 Respect the 10k–50k-person scale

A literal population of 50,000 is not a compressed Britain unless TCE explicitly makes it one.

There are two defensible approaches:

**A connected regional simulation.** Model individual people locally and represent external settlements more coarsely, with explicit trade, migration, knowledge, and financial links. This is my preferred approach for historically plausible industrial districts.

**A closed small world.** Keep all people individual, but accept that market size, specialization, resource diversity, and opportunities for invention differ from historical national economies. Calibrate that world on its own terms.

Do not quietly provide a small settlement with the purchasing power, inventors, and skilled suppliers of millions of unseen people.

## 5.6 Make non-industrialization genuinely possible

There is an important infinite-horizon issue: **if every technology retains a fixed positive discovery probability forever, eventual discovery becomes almost certain.** Random delays alone do not create persistent non-industrialization.

Allow enduring states in which relevant experimentation receives no resources, complementary capabilities are inaccessible, or adoption remains unprofitable. A society can also discover a machine and rationally never deploy it.

Possible outcomes should include prosperous labor-intensive commerce, limited water-powered industry, dependence on imported machinery, a failed state industrial project, and sustained industrial expansion. These are proposed outcome classes, not mandatory historical stages.

## 5.7 Computation and validation

Use multiple update rates: daily or shift-level labor and production; monthly orders, wages, credit, and investment; slower institutional change and advanced training. Process experimentation as sparse projects rather than checking every person against every technology every frame.

Cache routes and local price information. Store machinery and production requirements as compact component data; reserve detailed individual simulation for decisions and activities where individual variation matters.

Validate **ensembles**, not one successful timeline. Useful counterfactual tests include removing local coal while retaining water power, lowering wages without changing machinery, cutting an import route, changing patent enforcement, removing a key mechanic, and funding inspectors without changing the law.

Measure both production and life outcomes: output per worker, machine replacement, power use, household consumption, working hours, unemployment, education, mortality, and institutional finances.

## 5.8 Existing models and games worth borrowing from

| Model or game | Useful contribution | What TCE must add or change |
| --- | --- | --- |
| **Dosi–Fagiolo–Roventini “Keynes meets Schumpeter” models** | Heterogeneous firms, innovation and imitation, capital goods, demand, and financial feedback. | Historical resource constraints, spatial logistics, detailed households, and institutions arising from individuals. [Iris](https://www.iris.sssup.it/handle/11382/302310) |
| **Ciarli–Lorentz–Savona–Valente micro-to-macro model** | Links production organization, consumption patterns, distribution, and structural change. | Explicit geography, physical machinery requirements, historical knowledge, and demographic detail. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1467-999X.2009.04069.x) |
| **Galor–Weil; Murphy–Shleifer–Vishny** | Population–technology feedback and coordination failures provide useful causal structures. | They are aggregate theories, not ready-made individual-agent industrialization engines. [National Bureau of Economic Research](https://www.nber.org/papers/w6811) |
| **Victoria 3** | Production methods connect buildings to changing inputs and occupations. | TCE should make adoption establishment-specific and endogenous rather than treating the production-method interface as the causal explanation. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-57-the-journey-so-far) |
| **Workers & Resources: Soviet Republic** | Citizen work, education, care, and physical production logistics demonstrate useful operational detail. | Replace central player planning with autonomous investors, households, institutions, and discovery processes. [Soviet Republic](https://www.sovietrepublic.net/) |

---

# 6. Sources, datasets, and uncertainty management

## 6.1 Calibration datasets

| Resource | Best use | Main caution |
| --- | --- | --- |
| **Maddison Project Database 2023** | Long-run population and GDP-per-person comparisons; coverage includes 169 countries and extends to 2022. | Early observations are sparse reconstructions, not uniformly measured annual data. Use the named version for reproducibility. [Groningen Research Portal](https://research.rug.nl/nl/datasets/maddison-project-database-2023/) |
| **Bank of England: A Millennium of Macroeconomic Data**, Thomas and Dimsdale | British output, wages, prices, capital, trade, and financial benchmarks. | Variables have different coverage and underlying sources; the dataset is a compilation, not one homogeneous measurement system. [Bank of England](https://www.bankofengland.co.uk/statistics/research-datasets) |
| **Allen’s historical wage, price, and agricultural datasets** | Relative factor prices, commodity baskets, and comparisons between locations. | Daily wages, annual earnings, household income, and different occupational samples are not interchangeable. [Nuffield College](https://www.nuffield.ox.ac.uk/people/sites/allen-research-pages/) |
| **Cambridge Group for the History of Population and Social Structure** | Urbanization, migration, occupational structure, and historical demography. | Match geographical boundaries, occupational definitions, and urban thresholds. [CAMPOP](https://www.campop.geog.cam.ac.uk/blog/2024/08/22/stuck-in-the-mud/) |
| **IISH Global Collaboratory on the History of Labour Relations; Clio-Infra** | Comparative labor arrangements and long-run social and economic indicators. Labor-relations benchmarks include 1500, 1650, 1800, 1900, and 2000. | Coverage and classification quality vary; inspect the contributing sources rather than treating a harmonized table as equally certain everywhere. [IISH](https://iisg.amsterdam/nl/onderzoek/projectclusters/global-economic-developments) |
| **Machine collections and establishment studies** | Actual materials, mechanisms, historical designs, power adoption, and firm-scale relationships. | Surviving machines are not a random sample of all installations or failed experiments. Pair museum evidence with business and establishment records. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/objects/co50900/newcomen-atmospheric-engine) |

## 6.2 Keep the major disagreements visible

The core reading should include **both sides of the central disputes**:

**Allen versus Humphries and Schneider** for induced invention and spinning wages; **North and Weingast alongside Sussman and Yafeh** for institutional timing; **Pomeranz alongside Broadberry, Guan, and Li** for the Great Divergence; and **Allen alongside Crafts** for wages, productivity, and distribution. These disagreements should become alternative parameterizations or model specifications, not disappear into one blended number. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/j.1468-0289.2010.00532.x)

Evidence is particularly thin for universal estimates of invention probability, failed-project costs, early machine reliability, maintenance burdens, knowledge-loss rates, and the independent numerical effects of “culture” or “institutions.” Historical records also disproportionately preserve successful enterprises, formal transactions, and visible elites.

For those quantities, use explicit priors, sensitivity analysis, and observable consequences. A model that reproduces the date of British industrialization but fails on adoption lags, regional divergence, household welfare, or machine replacement is not well calibrated.

**The strongest TCE implementation is therefore not an Industrial Revolution event. It is a set of people and establishments discovering that new techniques sometimes pay—and building, or failing to build, the networks that let those techniques survive and spread.**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92869-20b4-83e9-8b58-7388c06cfc94)
