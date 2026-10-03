# Electric power, gas, and modern utilities: a simulation-ready report for TCE

**Recommendation:** model utilities as **energy services delivered through physical networks, purchased by households and firms, and operated by institutions**. Do not represent electricity as a citywide coverage flag, or gas as a mandatory step toward electrification.

TCE needs four interacting layers: demand for services such as lighting and heating; appliances that convert energy into those services; networks with local capacity and reliability constraints; and organizations that finance construction, collect revenue, maintain assets, and allocate shortages. This follows the historical distinction between consuming energy and obtaining useful services from it—a distinction that becomes especially important when technologies change radically. [Sage Journals](https://journals.sagepub.com/doi/abs/10.5547/ISSN0195-6574-EJ-Vol27-No1-8)

The resulting city can contain electric factories, gas-lit streets, homes using oil lamps, and neighborhoods without reliable connections simultaneously. Progress should mean better services becoming affordable and dependable—not every building advancing through the same sequence.

---

## 1. Mechanisms: causal rules TCE can implement

### 1.1 Demand arises from activities and equipment, not population alone

People want illumination, cooked food, comfortable temperatures, mechanical work, refrigeration, and communication. Electricity and gas are intermediate inputs. Historical research on lighting shows that changes in appliances, fuels, infrastructure, and prices jointly transformed consumption; extrapolating fuel use alone misses much of the transition. [Sage Journals](https://journals.sagepub.com/doi/abs/10.5547/ISSN0195-6574-EJ-Vol27-No1-8)

**Implementable rule:** each household, enterprise, and public institution requests services, then selects among available equipment and fuels.

| Service | Useful output to represent | Examples of competing provision |
| --- | --- | --- |
| Lighting | Lumen-hours, with minimum illumination for particular activities | Oil lamp, gas burner, incandescent lamp, efficient electric lamp |
| Cooking | Meals or useful cooking heat | Biomass stove, coal range, gas burner, electric cooker |
| Space conditioning | Occupied hours within an acceptable temperature range | Hearth, boiler, district heat, resistance heater, heat pump |
| Mechanical work | Shaft-work or machine-hours | Human/animal power, waterwheel, steam engine, electric motor |
| Refrigeration | Storage capacity maintained below a temperature threshold | Ice delivery, electrically powered refrigeration |

These are proposed TCE abstractions, not claims that households explicitly optimize engineering quantities.

For a building:

\[
P\_b(t)=\sum\_a n\_{b,a}\,p\_a\,u\_{b,a}(t)
\]

Here \(n\_{b,a}\) is appliance count, \(p\_a\) is rated electrical power, and \(u\_{b,a}(t)\) represents operation or duty cycle. Occupancy, working shifts, daylight, weather, and household routines determine operation.

This structure matters because **a connected household without appliances has little demand**, whereas one factory can dominate a small town’s load. RAMP provides an existing bottom-up, stochastic approach to generating multi-energy demand from equipment and user behavior. [Journal of Open Source Software](https://joss.theoj.org/papers/10.21105/joss.06418)

### 1.2 Adoption requires affordability, complementary equipment, and confidence in service

A connection is an investment distinct from buying energy. Households may need internal wiring, a meter, a deposit, compatible lamps or motors, and permission to alter a rented building.

Experimental evidence from rural Kenya found that connection demand declined strongly with price, newly connected households consumed little electricity, and measured medium-run benefits were limited. This is strong evidence against assigning a large automatic productivity bonus to every new connection. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/705417)

**Implementable rule:** compare alternatives using:

\[
\text{generalized service cost}
=
\text{energy cost}
+\text{annualized equipment and connection cost}
+\text{time cost}
+\text{expected interruption cost}.
\]

An agent adopts only when financing, equipment supply, skills, and expected benefits permit. Retain existing appliances until replacement becomes worthwhile; allow households to keep multiple fuels.

For TCE, this produces useful transitions without scripting them: electric lighting can replace gas lighting while an existing gas cooker remains economical; a shop may purchase a generator before the surrounding households can afford connections.

### 1.3 Networks expand when sufficient demand can support fixed costs

A utility must pay for a distribution route before all prospective customers join it. Demand concentrated along a street or around an industrial customer can spread those costs over more sales.

The Kenyan electrification experiment identified substantial economies of scale in distribution construction. Global historical research also emphasizes the role of finance and enterprise organization in spreading electricity internationally. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/705417)

**Implementable rule:** evaluate candidate extensions using expected connected demand, rather than nearby population alone:

\[
NPV\_{\text{extension}}
=
-\text{construction cost}
+\sum\_y
\frac{\text{expected collected revenue}-\text{operating cost}}
{(1+r)^y}.
\]

A private operator may require a positive financial return. A municipality may add public-lighting, industrial-development, or redistribution benefits. A cooperative may accept a long payback because its users also own the assets.

Construction cost should come from TCE’s own quantities:

\[
C\_{\text{extension}}
=
L(c\_{\text{excavation}}+c\_{\text{conductor/pipe}}+c\_{\text{restoration}})
+C\_{\text{equipment}}+C\_{\text{connections}}+C\_{\text{rights}}.
\]

This lets street width, rock, existing paving, local wages, metal prices, and rights-of-way matter without importing an arbitrary modern dollar cost into an alternative civilization.

### 1.4 Gas and electricity require different physical networks

**Manufactured gas** was a production industry, not simply natural gas extracted from the ground. Coal-gas systems required gasmaking equipment, cleaning and purification, storage holders, distribution pipes, and consumer fittings. Coke and other by-products affected the economics. Historic England’s research documents substantial technological variation within this industry. [Historic England](https://historicengland.org.uk/research/results/reports/182-2020)

A useful TCE chain is:

**Feedstock → gasworks → purification → gas holder → street mains → meter → compatible burner.**

**Natural gas** instead requires production or imported supply, processing, transmission pipelines, compression where necessary, city-gate regulation, distribution mains, and service connections. Storage helps reconcile relatively steady production with variable demand. [U.S. Energy Information Administration](https://www.eia.gov/energyexplained/natural-gas/delivery-and-storage.php)

**Electricity** requires generation, voltage transformation where applicable, transmission, distribution, and building connections. Higher-voltage transmission reduces the losses associated with moving substantial power over distance; transformation connects different voltage levels. [U.S. Energy Information Administration](https://www.eia.gov/energyexplained/electricity/delivery-to-consumers.php)

**Implementable rule:** use shared graph infrastructure, but give each carrier its own constraints.

| Carrier | Important state and constraints |
| --- | --- |
| Electricity | Instantaneous power balance, conductor and transformer capacity, losses, voltage quality, compatible electrical standards |
| Gas | Stored quantity, source throughput, pipe capacity, pressure adequacy, leakage, fuel composition and appliance compatibility |
| District heat | Heat-source capacity, temperature, circulation, pipe losses, connected heat exchangers |
| Water | Source yield, stored volume, pressure, pumping power, quality |

Do not treat electricity as an ordinary warehouse commodity. Conversely, do not treat gas as entirely instantaneous: gas holders, storage facilities, and gas already within pressurized networks can buffer supply.

### 1.5 Adequate generation does not guarantee local service

A town can have enough total generation while a particular transformer or feeder cannot supply its neighborhood. This is not merely a theoretical distinction: in July 2024, Eskom described local load reduction to protect overloaded equipment while nationwide load-shedding remained suspended. Its attribution of particular overloads should be treated as the utility’s account, not a complete explanation of every local failure. [Eskom](https://www.eskom.co.za/eskom-resumes-load-reduction-as-loadshedding-remains-suspended-to-protect-human-life-as-a-result-of-electricity-theft/)

**Implementable rule:** a building receives service only when all necessary conditions hold:

\[
\text{served}
=
\text{connected}
\land \text{source available}
\land \text{network feasible}
\land \text{account authorized}
\land \text{usable quality}.
\]

Track at least four distinct interruption causes: generation or fuel shortage; local network overload; damaged equipment; and financial or administrative disconnection. They require different remedies.

A new power station should not repair a broken service cable. A larger transformer should not solve a coal shortage.

### 1.6 Reliability emerges from assets, hazards, and repair organizations

Outages are spatially correlated because buildings share assets, and weather can damage many assets simultaneously. US outage data show that major events dominate much of the year-to-year variation in interruption duration. [U.S. Energy Information Administration](https://www.eia.gov/todayinenergy/detail.php?id=66744)

**Implementable rule:** generate failures at assets or common-cause hazard footprints—not independently for each citizen.

For each asset, maintain condition, loading history, environmental exposure, inspection history, and repair requirements. A proposed hazard structure is:

\[
\lambda\_e(t)=
\lambda\_{\text{type}}
f\_{\text{condition}}
f\_{\text{loading}}
f\_{\text{exposure}}.
\]

The functions require calibration; they are not universal empirical laws. Add separate storm, flood, fire, conflict, and excavation events that can affect multiple assets together.

Restoration should require fault detection, access, available crews, materials, and completion of work. Switching around damage can restore customers before the failed asset itself is repaired.

Record customer-weighted reliability:

\[
SAIDI=\frac{\sum\_j N\_j d\_j}{N\_{\text{customers}}},
\qquad
SAIFI=\frac{\sum\_j N\_j}{N\_{\text{customers}}}.
\]

Here \(N\_j\) is the number of customers interrupted by event \(j\), and \(d\_j\) its duration. These indices summarize interruption duration and frequency; they do not measure whether unconnected people have access. [U.S. Energy Information Administration](https://www.eia.gov/todayinenergy/detail.php?id=66744)

### 1.7 Ownership changes incentives and financing—not electrical physics

The World Bank’s comparative work rejects a single universally successful reform template. Private generation investment, public utilities, and hybrid arrangements have produced different results under different conditions; well-governed public utilities can perform effectively. [World Bank](https://www.worldbank.org/en/topic/energy/publication/rethinking-power-sector-reform)

TCE should therefore permit the following institutional configurations rather than giving ownership types fixed efficiency bonuses:

| Institution | Primitives to represent |
| --- | --- |
| Municipal utility | City ownership; tariff revenue; municipal borrowing; transfers from taxes; elected or appointed priorities |
| Private franchise | Investor ownership; concession territory and duration; tariff rules; service obligations; dividends; possible revocation |
| State or regional utility | Regional finance and planning; cross-subsidies; strategic industrial supply; multiple municipalities |
| Cooperative | Member capital; member governance; shared connection costs; surplus returned or reinvested |
| Mixed system | Separate asset owner, operator, generators, and retail supplier; negotiated access and settlement |

Maintain a real utility ledger:

**Collected bills + subsidies + borrowing − fuel/imports − wages − maintenance − debt service − investment.**

Insufficient revenue can delay maintenance or expansion; poor service can reduce willingness to connect or pay. Political authorities can interrupt that feedback through subsidy, restructuring, enforcement, or institutional replacement.

Separate **technical losses**, **unbilled consumption**, and **unpaid bills**. Electricity consumed without payment still loads the network; an unpaid bill is not additional physical energy loss.

---

## 2. Parameters: measured benchmarks and proposed starting values

**Confidence convention:** **H** means a well-defined official statistic, published framework threshold, or documented installation value; **M** means a historical reconstruction or limited-context observation. Neither means “universally transferable.” **Design prior** means a proposed TCE setting, not a historical estimate.

### 2.1 Electricity service levels

The World Bank/ESMAP Multi-Tier Framework is a useful starting point because it separates connection from service adequacy. Selected household thresholds are below. Power and daily-energy entries are alternative capacity criteria, **not estimates of actual consumption**. [UNCC Learn](https://www.uncclearn.org/wp-content/uploads/library/beyond.pdf)

| Attribute | Tier 1 | Tier 2 | Tier 3 | Tier 4 | Tier 5 |
| --- | --- | --- | --- | --- | --- |
| Power capacity | 3 W | 50 W | 200 W | 800 W | 2,000 W |
| Daily energy capacity | 0.012 kWh | 0.2 kWh | 1 kWh | 3.4 kWh | 8.2 kWh |
| Availability | 4 h/day | 4 h/day | 8 h/day | 16 h/day | 23 h/day |
| Evening availability | 1 h | 2 h | 3 h | 4 h | 4 h |

All entries: **H as framework definitions**. Additional reliability criteria include at most 14 interruptions/week for Tier 4; Tier 5 requires at most three, totaling less than two hours/week. The affordability test uses a 365-kWh/year package costing less than 5% of household income. The full framework also considers quality, legality, and safety. Tier 5 is not a specification for every modern household or critical facility. [UNCC Learn](https://www.uncclearn.org/wp-content/uploads/library/beyond.pdf)

For TCE, retain the underlying attributes rather than storing only one tier number. An evening-only microgrid and an unreliable nominally continuous connection should remain distinguishable.

### 2.2 Demand per capita: useful calibration bounds, not household recipes

The following World Bank/IEA values measure **national electricity consumption across sectors divided by population**. They include industrial and commercial demand. They are not residential consumption and should not be assigned directly to individual citizens. Years are intentionally explicit because the series have different observation dates. [World Bank Open Data](https://data.worldbank.org/indicator/EG.USE.ELEC.KH.PC?locations=M1)

| Country | Year | Electricity, kWh/person/year | Confidence |
| --- | --- | --- | --- |
| Nigeria | 2023 | 144 | H for the reported national series |
| India | 2023 | 1,182 | H |
| Brazil | 2024 | 3,068 | H |
| South Africa | 2023 | 3,247 | H |
| China | 2023 | 6,524 | H |
| Japan | 2024 | 7,530 | H |
| Qatar | 2023 | 19,963 | H |

Sources: World Development Indicators, **EG.USE.ELEC.KH.PC**, drawing on IEA statistics. [World Bank Open Data](https://data.worldbank.org/indicator/EG.USE.ELEC.KH.PC?locations=C8)

The modeling implication is not that nationality determines demand. It is that one universal “modern citizen uses X” constant cannot cover different combinations of access, income, industrial structure, climate, and equipment.

### 2.3 Household, historical, and reliability benchmarks

| Parameter | Dated value or range | Appropriate interpretation | Source; confidence |
| --- | --- | --- | --- |
| US residential electricity purchases | **10,791 kWh/customer/year**, 2022 | Utility-account average, not per person; excludes electricity supplied directly by household generation | EIA; **H**. [U.S. Energy Information Administration](https://www.eia.gov/tools/faqs/faq.php?id=97&t=3) |
| Variation between US state residential averages | **6,178–14,774 kWh/customer/year**, 2022 | Hawaii to Louisiana; illustrates regional variation, not an individual-household range | EIA; **H**. [U.S. Energy Information Administration](https://www.eia.gov/tools/faqs/faq.php?id=97&t=3) |
| Great Britain domestic benchmark | **2,700 kWh electricity + 11,500 kWh gas per household/year**, 2023 benchmark | Regulatory “typical consumption” values, not a current universal household average | Ofgem; **H as benchmark**. [Ofgem](https://www.ofgem.gov.uk/cy/press-release/energy-prices-fall-again-winter) |
| Historical gas-light consumption example | **5 ft³/hour ≈ 0.142 m³/hour** | One contemporary engineering example; burner and gas composition matter | Keene, 1918; **M for transferability**. [Project Gutenberg](https://www.gutenberg.org/files/50846/50846-h/50846-h.htm) |
| Early Calcutta generating station | **1,000 kW**, **450/225 V DC**, April 1899 | An actual early urban supply installation, not a universal plant size | IEEE historical milestone; **H**. [ETHW](https://ethw.org/Milestones%3ACalcutta_Electric_Supply_Corp%2C_1899) |
| US transmission and distribution losses | Approximately **5%**, 2018–2022 | Modern national reference; not a historical-network assumption | EIA; **H**. [U.S. Energy Information Administration](https://www.eia.gov/tools/faqs/faq.php?id=105&t=3) |
| US customer interruption duration | Approximately **11 hours/customer/year**, 2024 | Includes major events | EIA; **H**. [U.S. Energy Information Administration](https://www.eia.gov/todayinenergy/detail.php?id=66744) |
| US customer interruption frequency | Approximately **1.5 interruptions/customer/year**, 2024 | Non-momentary interruptions | EIA; **H**. [U.S. Energy Information Administration](https://www.eia.gov/todayinenergy/detail.php?id=66744) |
| US interruption duration excluding major events | Approximately **2 hours/customer/year**, routinely | Useful ordinary-failure calibration distinct from disasters | EIA; **H for the stated aggregation**. [U.S. Energy Information Administration](https://www.eia.gov/todayinenergy/detail.php?id=66744) |

**Gas-volume warning:** manufactured gas and natural gas are not interchangeable quantities. Keene’s manual gives different calorific values for different manufactured gases. Modern gas accounting also depends on heating value and the conditions used to define volume. Store gas energy content explicitly; distinguish higher/lower heating-value conventions wherever efficiency calculations require it. [Project Gutenberg](https://www.gutenberg.org/files/50846/50846-h/50846-h.htm)

Using the historical lamp example, two lamps operated four hours daily would consume approximately **413 m³/year**. That is a calculated lighting-only scenario—not an observed average household gas bill.

### 2.4 Transparent design priors

These values are suitable for sensitivity testing before TCE has calibrated equipment catalogs. **They are not measured historical ranges.**

| Proposed test parameter | Starting range | Purpose |
| --- | --- | --- |
| Aggregate annual load factor: average/peak power | **0.35–0.65** | Test how schedules and demand diversity change required peak capacity |
| Dispatchable firm-capacity reserve target | **15–25% above forecast peak** | Test investment conservatism; not a renewable nameplate-capacity rule |
| Simple-network loss sensitivity | **5–15% of upstream energy** | Early approximation before losses depend on individual assets |
| Utility scheduling interval | **15–60 simulation minutes** | Balance load-shape resolution and cost |
| Ordinary versus exceptional repair scenarios | **1–8 hours; 12–72 hours; multi-day disaster cases** | Test crew queues and correlated damage, not impose universal repair times |

Once equipment types are authored, replace these aggregate assumptions with network geometry, capacity, condition, and actual repair jobs.

### 2.5 Worked sizing example

Assume—not predict—a TCE town with **20,000 people**, residential use of **1,000 kWh/person/year**, and another **10 GWh/year** consumed by businesses and public facilities.

Total delivered demand is **30 GWh/year**, averaging **3.42 MW**. At a load factor of **0.5**, peak delivered demand is approximately **6.85 MW**. With losses equal to **8% of upstream supply**, the corresponding upstream peak is approximately **7.45 MW**. A **20% firm reserve** gives an illustrative target of **8.94 MW**.

This does **not** imply that 8.94 MW of solar nameplate capacity provides equivalent firm service. Nor does adequate generation eliminate feeder constraints.

---

## 3. Historical and regional variation

### 3.1 Treat eras as capability regimes, not mandatory stages

| Capability regime | Representation in TCE |
| --- | --- |
| Forager starting conditions | Light, cooking, and warmth exist without utility networks. Provision is carried fuel, equipment, labor, and shelter—not zero energy consumption. |
| Early farming | More permanent buildings and workshops can support shared facilities; households still need not use network energy. |
| Pre-industrial cities | Public lighting can be a labor-intensive purchased service. Specialized fuel and mechanical-power systems may exist without general electrification. |
| Industrial capabilities | Gasmaking, generators, conductors, meters, motors, and network finance enable new combinations; public and private systems can coexist. |
| Modern capabilities | Interconnected electricity, gas imports and storage, district energy, distributed generation, electronic controls, and backup systems become possible—not compulsory. |

These are scenario categories. They should not classify every real society into a single technological stage.

Localized natural-gas production and bamboo transport around Sichuan’s salt industry provide a particularly useful counterexample to a Europe-centered sequence: useful gas infrastructure need not begin with nineteenth-century municipal coal-gas lighting. [AAPG | Advancing Geoscience Since 1917](https://www.aapg.org/news-and-media/explorer/big-drilling-in-ancient-china/)

### 3.2 Dated milestones—not universal “firsts”

| Place and period | Documented development | TCE implication |
| --- | --- | --- |
| London, **1807–1842** | Pall Mall demonstration in 1807; Gas Light and Coke Company formed in 1812; gas spread across major streets over subsequent decades | Demonstration, enterprise formation, and broad service are different events. [London Museum](https://www.londonmuseum.org.uk/collections/london-stories/gas-lamps-illuminated-london/) |
| Rio de Janeiro, **1854–1857** | Gas service began in 1854; the operator’s history records **3,027 public lamps and 3,200 residences** by 1857 | Public procurement and household subscriptions can expand together but remain separate markets. [Naturgy](https://www.naturgy.com.br/corporativo/quem_somos/nossa_compania/presenca-no-brasil/historia-rio/) |
| Tokyo, **1874–1885** | Gas service began in 1874; the initially public enterprise was privatized in 1885 | Ownership can change while the physical network persists. [Tokyo Gas](https://www.tokyo-gas.co.jp/letter/2020/12/20201223-3.html) |
| Kimberley, South Africa, **1882** | Sixteen electric arc street lamps were operating by September | A mining town can adopt electricity early without waiting for universal national development. [Eskom](https://www.eskom.co.za/heritage/the-early-years/) |
| New York, **1882**, and the US, **1940** | Pearl Street opened in September 1882; **78.7% of occupied US homes** had electric lighting in 1940 | Early commercial supply and widespread domestic adoption can be separated by generations. [Census.gov](https://www.census.gov/about/history/stories/monthly/2026/september-2026.html) |
| Calcutta/Kolkata, **1899** | Commercial supply included a 1-MW DC generating installation | Imported technology, licensed enterprise, and concentrated urban demand can produce geographically selective electrification. [ETHW](https://ethw.org/Milestones%3ACalcutta_Electric_Supply_Corp%2C_1899) |
| Britain, **1960s–1970s**; Rio, **1998–2007** | Manufactured-gas systems were converted to natural gas on very different regional schedules | Fuel substitution is a network-and-appliance conversion project, not an instantaneous resource swap. [Historic England](https://historicengland.org.uk/research/results/reports/182-2020) |

Regional differences should emerge from resources, climate, density, trade, finance, and political priorities. A hydro-rich region should not need a coal-gas industry before generating electricity. A dense cold city can support district heat; another city may favor individual boilers or electric equipment.

District heating illustrates the scale of regional specialization: in **2018**, it supplied less than **6% of global heat**, while **China and Russia each accounted for more than one-third of global district-heat production**. It is important in particular systems, not a universal modern destination. [IEA](https://www.iea.org/articles/how-can-district-heating-help-decarbonise-the-heat-sector-by-2024)

### 3.3 Electrification changes opportunities, but outcomes depend on context

South African quasi-experimental research found that rural electrification increased female employment, with mechanisms involving changes in household production and employment opportunities. The Kenyan experiment found much more limited medium-run effects. These are different settings and interventions, not interchangeable estimates. [AEA Publications](https://pubs.aeaweb.org/doi/10.1257/aer.101.7.3078)

**TCE implication:** electricity should release time or enable production only where an actual appliance, production process, market, or institution uses it. A motor without customers, a connection without equipment, or evening light without a feasible activity should not automatically increase income.

---

## 4. Stylized facts and validation targets

A correct model should reproduce the following patterns under appropriate conditions—not force every world to reproduce the same history.

| Pattern | Evidence or numerical anchor | Simulation test |
| --- | --- | --- |
| **Infrastructure precedes universal uptake** | US domestic electric lighting remained below universal coverage in 1940, decades after early urban supply. [Census.gov](https://www.census.gov/about/history/stories/monthly/2026/september-2026.html) | Permit mains alongside unconnected or minimally equipped homes. |
| **Service consumption can grow much faster than energy efficiency improves** | Fouquet and Pearson estimate UK lighting prices fell below **1/3,000** of their 1800 level by 2000, while per-capita lighting services rose **6,500-fold**. [Sage Journals](https://journals.sagepub.com/doi/abs/10.5547/ISSN0195-6574-EJ-Vol27-No1-8) | Cheaper illumination increases usage, but allow saturation; do not hardcode those historical multipliers. |
| **Different cities have radically different electricity intensity** | The national benchmarks above span more than two orders of magnitude. [World Bank Open Data](https://data.worldbank.org/indicator/EG.USE.ELEC.KH.PC?locations=C8) | Vary equipment, access, industry, and climate; population alone must not determine demand. |
| **Local shortages coexist with system-wide adequacy** | Eskom’s July 2024 distinction between local load reduction and national load-shedding. [Eskom](https://www.eskom.co.za/eskom-resumes-load-reduction-as-loadshedding-remains-suspended-to-protect-human-life-as-a-result-of-electricity-theft/) | An overloaded transformer interrupts its district even with spare generation elsewhere. |
| **Rare events dominate some reliability outcomes** | Major events accounted for **80% of US interruption hours in 2024**. [U.S. Energy Information Administration](https://www.eia.gov/todayinenergy/detail.php?id=66744) | Add correlated disasters; ordinary independent failures should not explain the entire outage distribution. |
| **Improved technology does not instantly erase legacy systems** | Rio’s manufactured-to-natural-gas conversion extended from 1998 to 2007. [Naturgy](https://www.naturgy.com.br/corporativo/quem_somos/nossa_compania/presenca-no-brasil/historia-rio/) | Preserve equipment compatibility, retrofit labor, financing, and staged conversion. |
| **Connections do not guarantee economic transformation** | Contrasting Kenyan and South African causal evidence. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/705417) | Benefits arise through actual household and production activities. |
| **Ownership and performance are not identical variables** | Comparative power-sector reform research finds multiple workable institutional arrangements. [World Bank](https://www.worldbank.org/en/topic/energy/publication/rethinking-power-sector-reform) | Changing ownership without changing finance, incentives, competence, or obligations should not magically alter performance. |

Also test three engineering invariants independently of historical calibration: energy conservation; no supply through disconnected paths; and no simultaneous allocation of the same storage energy to multiple consumers.

---

## 5. Modeling recommendation for TCE

### 5.1 Put electrical and gas connections on buildings, not individual people

A person needs access to services, but normally does not require a separate network node.

| Entity | Minimum useful state |
| --- | --- |
| Person | Activities, location, service needs, household membership, occupation |
| Household or enterprise | Budget, account, equipment stock, service preferences, arrears, backup options |
| Building | Connections, internal capacity, occupancy, thermal characteristics, delivered-service history |
| Network asset | Endpoints, carrier, capacity, compatibility, condition, failure state, owner |
| Generator/gasworks/heat plant | Conversion recipe, operating limits, fuel stock, staff, maintenance state |
| Utility institution | Assets, tariffs, contracts, finances, obligations, planning policy, repair organization |
| Incident/repair job | Cause, affected assets, required crew/materials, access, progress |

This preserves individual consequences while avoiding a utility-flow solver with 50,000 independent person nodes.

### 5.2 Start with radial electricity distribution and explicit bottlenecks

For the first utility implementation, use an **operationally radial distribution graph**. Spare ties can exist but remain open until switching changes the topology.

A tree sweep aggregates downstream **net** demand, including local generation. Another pass applies transformer and conductor limits, allocates available supply, and estimates losses and voltage adequacy. Reverse flows should remain possible where distributed generation exceeds local demand.

This is much cheaper than solving detailed alternating-current behavior everywhere. General-purpose tools such as pandapower provide power-flow methods and network models suitable for checking representative cases offline. [pandapower](https://www.pandapower.org/about/)

**Do not use unrestricted maximum flow as though it were a faithful model of a meshed electrical grid.** Electrical flows obey circuit constraints; operators cannot freely route each kilowatt along arbitrary paths. For a later interconnected transmission model, use an appropriate linearized power-flow approximation, or retain a clearly labeled transport abstraction.

For gas, begin with capacity- and pressure-adequacy checks at district scale, explicit storage, and leakage. Validate more detailed pressure-flow behavior against a gas-network solver before presenting it as engineering-realistic. GasLib supplies benchmark networks and operating constraints for this purpose, but represents modern gas-network problems rather than historical town-gas demand. [GasLib](https://gaslib.zib.de/)

### 5.3 Preserve conservation, operational limits, and shortages

At each electricity interval:

\[
G + I + B\_{\text{discharge}}
=
D\_{\text{served}}+L+B\_{\text{charge}}+X.
\]

Generation \(G\), imports \(I\), exports \(X\), network losses \(L\), and battery flows must use consistent units. Storage has separate **energy capacity** and **power limits**, with charging/discharging losses.

Available plant output should depend on fuel, staffing, condition, weather or water where relevant, and operating constraints. Nameplate capacity is not guaranteed output.

When supply is insufficient, the utility applies an authored allocation policy: emergency services first, contractual priority, rotating district cuts, proportional rationing, political favoritism, or price-mediated reduction. The policy belongs to the institution.

External connections must also have capacity, price, reliability, and counterparty availability. They should not be infinite emergency suppliers.

### 5.4 Track service outcomes, not generic penalties

Use a rolling service record for each building:

* requested and delivered energy;
* peak power available;
* hours available, including evening hours;
* interruptions and quality failures;
* expenditure and connection status.

Then translate service into specific consequences. Lighting changes feasible activities; unavailable motor power stops particular machines; loss of refrigeration affects stored goods; heating failure changes indoor temperature.

Avoid immediate universal penalties such as “every building loses 30% productivity during an outage.” The same interruption should affect a bakery, a grain warehouse, and a household differently.

Cross-utility dependencies should include buffers. A pump outage need not stop water delivery until stored water is exhausted. A communications site with a battery should continue until its reserve is depleted. A combined heat-and-power plant must account for both outputs through one conversion process, not receive two independent free capacities.

### 5.5 Use different time scales without losing peak demand

A proposed scheduling structure is:

| Process | Suggested cadence |
| --- | --- |
| Failure, disconnection, switching, repair completion | Event-driven |
| Demand and dispatch | Every 15–60 simulation minutes |
| Billing and financial collection | Periodic account cycle |
| Equipment adoption and replacement | Household/enterprise decision events |
| Inspection and maintenance planning | Periodic plus condition-triggered |
| Major expansion | Investment and political decision events |

In fast-forward, use representative daily and seasonal demand profiles, but preserve fuel balances, maintenance accumulation, extreme-weather events, and outage consequences. Annual average demand alone cannot reveal an evening peak or a cold-weather capacity shortage.

For radial networks, a fixed number of graph sweeps scales approximately with asset count. That makes this architecture plausible for TCE, but **no measured Rust/UE5 performance benchmark has been established here**. Profile the actual network and event workload rather than assuming population count predicts solver cost.

### 5.6 Borrow selectively from city builders and engineering models

| Reference | Useful feature to borrow | Limitation or adaptation |
| --- | --- | --- |
| **Cities: Skylines** | Legible network coverage and supply feedback | Its electricity zones are too coarse for TCE’s local capacity and institutional questions. |
| **Cities: Skylines II**, documented 2023 design | Voltage classes, transformer limits, network bottlenecks, weather-sensitive demand, fuel deliveries, batteries | Roadside connections are too automatic for historical adoption. TCE should explicitly build connections and bound external trade. The developer diary describes game abstractions, not a validated power-flow model. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/electricity-water) |
| **Workers & Resources: Soviet Republic** | Utilities integrated with construction, production chains, logistics, and seasonal heat demand | Borrow operational dependencies, not the assumption of one player-controlled planning authority. TCE needs competing households, firms, and public bodies. [Steam Store](https://store.steampowered.com/app/784150/) |
| **RAMP** | Equipment- and activity-based stochastic multi-energy profiles | Adapt the demand-generation approach to TCE’s own agents rather than importing fixed modern lifestyles. [Journal of Open Source Software](https://joss.theoj.org/papers/10.21105/joss.06418) |
| **pandapower** | Reference electrical networks and power-flow calculations | Best used initially for offline validation of the cheaper runtime approximation. [pandapower](https://www.pandapower.org/about/) |
| **GasLib** | Gas-network topology and operational benchmark problems | Useful for solver testing, not for calibrating nineteenth-century household consumption. [MDPI](https://www.mdpi.com/2306-5729/2/4/40) |

**The minimum worthwhile post-v1 scope** is therefore building connections, equipment-based demand, local network capacities, supply and storage, utility accounts, and event-driven failures with actual repair work. Detailed electromagnetic transients, individual gas-pipe fluid dynamics, and wholesale-market bidding can wait.

---

## 6. Sources, datasets, and limits of the evidence

### Core research and reference material

**Historical services and adoption:** Fouquet and Pearson, *Seven Centuries of Energy Services: The Price and Use of Light in the United Kingdom (1300–2000)*, *The Energy Journal* 27(1), 2006. This is the strongest source here for why service quantities and service prices matter more than a simple fuel-consumption ladder. [Sage Journals](https://journals.sagepub.com/doi/abs/10.5547/ISSN0195-6574-EJ-Vol27-No1-8)

**Global institutional history:** Hausman, Hertner, and Wilkins, *Global Electrification: Multinational Enterprise and International Finance in the History of Light and Power, 1878–2007*, Cambridge University Press, 2008. Useful for situating urban networks within international finance and enterprise rather than treating electrification as isolated invention. [Cambridge University Press](https://www.cambridge.org/core/books/global-electrification/45568DA706AAEE519249B67E00CA2980)

**Manufactured gas:** Russell Thomas, *The Manufactured Gas Industry*, Historic England Research Report 182/2020; and E. S. Keene, *Mechanics of the Household*, 1918. The former is the broad technical-historical reference; the latter supplies contemporary appliance examples, which should not be mistaken for population averages. [Historic England](https://historicengland.org.uk/research/results/reports/182-2020)

**Service measurement and institutions:** Bhatia and Angelou, *Beyond Connections: Energy Access Redefined*, ESMAP, 2015; World Bank, *Rethinking Power Sector Reform in the Developing World*. Use these for multidimensional service assessment and institutional comparison. [UNCC Learn](https://www.uncclearn.org/wp-content/uploads/library/beyond.pdf)

**Causal outcomes:** Lee, Miguel, and Wolfram, *Experimental Evidence on the Economics of Rural Electrification*, *Journal of Political Economy* 128(4), 2020; Dinkelman, *The Effects of Rural Electrification on Employment: New Evidence from South Africa*, *American Economic Review* 101(7), 2011. Their differences are a reason to model mechanisms, not assign one universal electrification effect. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/705417)

**Calibration datasets:** World Bank/IEA electricity-consumption series; EIA household electricity, system-loss, and reliability statistics; MTF household surveys; RAMP demand-model inputs; pandapower and GasLib benchmark networks. Demand, service quality, and network feasibility require different datasets. [World Bank Open Data](https://data.worldbank.org/indicator/EG.USE.ELEC.KH.PC?locations=M1)

### What should remain uncertain

**Historical demand is the thinnest quantitative area.** This review did not establish a globally comparable pre-1900 household gas or electricity consumption series. Use dated installations, customers, lamps, equipment ratings, and operating hours to construct local scenarios; do not invent a universal industrial-era kWh-per-person value.

**Priority dates are definition-sensitive.** An experimental lamp, a private factory installation, public street lighting, commercial distribution, and widespread household service are different milestones. The timeline deliberately avoids unsupported “world’s first” claims.

**Performance comparisons need matched definitions.** Major-event inclusion can substantially change outage statistics. Household accounts differ from households and people. National consumption mixes sectors. Regulatory service thresholds are not observed consumption averages.

**Ownership and economic effects are context-dependent.** The evidence does not justify a permanent “private is efficient,” “public is equitable,” or “electrification increases productivity by X%” coefficient.

**Bottom line:** TCE’s most convincing utility history will emerge from **what people are trying to do, what equipment they own, what their networks can actually deliver, and whether their institutions can sustain the service**. That is sufficient to generate selective electrification, gas-to-electric transitions, competing supply systems, underinvestment, blackouts, recovery, and widening expectations—without scripting an industrial era.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556-tce-research/c/6ab92947-1ae4-83ea-bcdc-4850b1a9e8c9)
