# Climate zones and stochastic weather for The Civilization Engine

## Executive recommendation

**Build climate as a hierarchy, not as a collection of biome-specific weather dice:**

**Planetary geography → monthly climate → persistent regional circulation → spatially coherent daily weather → soil, snow and water storage → consequences for people.**

For TCE, I recommend a **geography-conditioned, multisite weather generator** built around Richardson/WGEN-style methods, with three extensions: regional weather regimes, explicit interannual variability, and a heavy-tail treatment for severe precipitation. Contemporary scientific generators demonstrate these components, although no single model automatically solves all of them for arbitrary fictional geography. [AGU Journals](https://agupubs.onlinelibrary.wiley.com/doi/10.1029/WR017i001p00182)

The crucial design distinction is that **Köppen classes describe long-term temperature and precipitation patterns; they do not uniquely specify the weather process producing them**. Two places with the same classification can have different storm types, drought persistence and rainfall intensity. Use Köppen as a diagnostic and a way to select initial parameter families—not as the causal engine. [HESS](https://hess.copernicus.org/articles/11/1633/2007/hess-11-1633-2007.pdf)

Throughout this report, **observations**, **published model parameters**, and **proposed TCE defaults** are distinguished. The proposed defaults are suitable for prototypes and sensitivity tests; they are not claimed to be globally calibrated empirical estimates.

---

## 1. Mechanisms: implementable causal rules

### 1.1 Geography establishes the climate envelope

Assume an Earthlike planet unless TCE explicitly varies rotation, orbit, axial tilt or atmospheric composition. Even then, the settlement map needs an **off-map climatic setting**: latitude, continental position, ocean temperatures and upwind terrain. Local elevation and distance to the nearest coast cannot uniquely determine climate. Earth’s circulation reflects rotation, seasonal heating and the distribution of continents and oceans. [NOAA](https://prod-01-alb-www-noaa.woc.noaa.gov/jetstream/global/global-atmospheric-circulations?utm_source=chatgpt.com)

A practical geographical generator should apply these rules:

| Driver | Implementable rule | Important qualification |
| --- | --- | --- |
| Latitude and season | Calculate day length and incoming solar energy by day of year; reverse seasonal phase between hemispheres. | Do not impose four equally important seasons everywhere. |
| Large-scale circulation | Initialize tropical convergence, subtropical subsidence, midlatitude westerlies and polar circulation as broad, seasonally moving patterns. | These are starting patterns, not rigid latitude boundaries. |
| Continentality | Increase seasonal temperature range where air has long continental fetch; reduce it and delay the seasonal response near oceans. | Upwind exposure matters more than straight-line coastal distance. |
| Monsoons | Allow seasonal land–ocean heating contrasts to change prevailing circulation and moisture delivery. | A monsoon needs onset, withdrawal and breaks—not merely higher summer rainfall. |
| Elevation | Apply a seasonally adjustable temperature lapse rate, then local terrain corrections. | Nighttime inversions can make valley floors colder than slopes. |
| Mountains and moisture | Advect moisture, condense some during ascent, transport cloud water, precipitate it, and deplete the remaining air mass. | Rain shadows require a moisture budget; a mountain must not create water. |

The circulation framework follows NOAA’s physical description. Ocean moderation and the coexistence of monsoon surges, squalls and local convection are particularly clear in Meteorological Service Singapore’s observations. [NOAA](https://prod-01-alb-www-noaa.woc.noaa.gov/jetstream/global/global-atmospheric-circulations?utm_source=chatgpt.com)

For solar geometry, FAO-56 provides inexpensive equations for solar declination, sunset hour angle, extraterrestrial radiation and day length. For example:

\[
N=\frac{24}{\pi}\cos^{-1}(-\tan\phi\tan\delta)
\]

where \(N\) is daylight hours, \(\phi\) latitude and \(\delta\) solar declination. Handle polar day and night explicitly rather than allowing an invalid inverse cosine. [FAOHome](https://www.fao.org/4/x0490e/x0490e07.htm)

A useful temperature downscaling equation is:

\[
\overline T\_{i,m}
=
T\_{\mathrm{regional},m}
-\Gamma\_m(z\_i-z\_{\mathrm{reference}})
+\Delta T\_{\mathrm{local},i,m}
\]

Here, the regional climate already contains continentality and ocean influences; local corrections represent effects such as aspect and cold-air pooling.

**Orographic precipitation:** Hergarten and Robl’s *Linear Feedback Precipitation Model*, LFPM 1.0, is an unusually relevant starting point. It represents transported water vapour and cloud water, conversion between them, precipitation loss and lateral dispersion. Unlike a simple slope multiplier, it includes long-range moisture depletion. The authors report computational cost scaling linearly with cell count. [GMD](https://gmd.copernicus.org/articles/15/2063/2022/)

For TCE, I would run a simplified moisture-transport calculation at world creation for several representative circulation patterns per month. Cache the resulting precipitation multipliers. Daily weather then chooses among those patterns rather than recalculating continental atmospheric transport every day.

### 1.2 Diagnose Köppen classes after generating monthly climate

Use twelve long-term monthly temperatures and precipitation totals. In a Peel-style implementation, **test aridity first**.

The annual aridity threshold, expressed directly in millimetres, is:

\[
R=20T\_{\mathrm{annual}}+c
\]

where \(c=280\) for strongly summer-dominated rainfall, \(0\) for strongly winter-dominated rainfall, and \(140\) otherwise; “strongly” means at least 70% in the relevant half-year.

| Classification test | Threshold |
| --- | --- |
| Desert, BW | Annual precipitation \(<R/2\) |
| Steppe, BS | Annual precipitation between \(R/2\) and \(R\) |
| Tropical, A, after excluding B | Coldest monthly mean \(\ge18^\circ\mathrm C\) |
| Rainforest, Af | Driest month \(\ge60\) mm |
| Monsoon, Am | Not Af; driest month \(\ge100-P\_{\mathrm{annual}}/25\) mm |
| Temperate versus continental, C/D | Peel uses \(0^\circ\mathrm C\) for the coldest-month boundary; other implementations use \(-3^\circ\mathrm C\) |
| Polar, E | Warmest month around or below \(10^\circ\mathrm C\) |
| Dry summer, “s” | Driest summer month \(<40\) mm and \(<1/3\) of wettest winter month |
| Dry winter, “w” | Driest winter month \(<1/10\) of wettest summer month |

These are classification definitions, not crop-response thresholds. Fix one convention, including exact-threshold ties, and test it against published examples. [HESS](https://hess.copernicus.org/articles/11/1633/2007/hess-11-1633-2007.pdf)

Do not change the underlying climate type whenever one unusual year occurs. More importantly, crops and vegetation should respond to continuous environmental variables, not abrupt classification boundaries.

### 1.3 Daily precipitation: occurrence, amount and persistence

Richardson’s original 1981 model generated precipitation with a Markov-chain/exponential process, then generated temperature and radiation jointly, conditioned on whether the day was wet. Later Richardson-type generators commonly use gamma or other distributions for positive precipitation amounts. [AGU Journals](https://agupubs.onlinelibrary.wiley.com/doi/10.1029/WR017i001p00182)

A convenient TCE parameterization uses:

* \(q\_m\): fraction of wet days in month \(m\);
* \(\rho\_m\): persistence of the wet/dry state;
* \(\mu\_{\mathrm{wet},m}\): mean precipitation on wet days;
* \(k\_m\): gamma shape.

For a locally stationary month:

\[
p\_{01}=q(1-\rho)
\]\[
p\_{11}=q+\rho(1-q)
\]

where \(p\_{01}\) is the probability of rain after a dry day and \(p\_{11}\) after a wet day. This parameterization gives stationary wet-day fraction \(q\), with \(p\_{11}-p\_{01}=\rho\).

The corresponding mean spell lengths are:

\[
E[L\_{\mathrm{dry}}]=\frac{1}{p\_{01}},
\qquad
E[L\_{\mathrm{wet}}]=\frac{1}{1-p\_{11}}
\]

For precipitation amount, a threshold-consistent implementation is:

\[
P\_t=
\begin{cases}
0,&\text{dry}\\
r\_0+\operatorname{Gamma}(k,\theta),&\text{wet}
\end{cases}
\]\[
\theta=\frac{\mu\_{\mathrm{wet}}-r\_0}{k}
\]

where \(r\_0\) is the wet-day threshold. Recode trace precipitation consistently when fitting this model.

The mean-water constraint is:

\[
E[P\_{\mathrm{month}}]
=
n\_mq\_m\mu\_{\mathrm{wet},m}
\]

This is an **expectation**, not a quota. Do not rescale every realized month back to its climatic normal.

**Worked synthetic example.** For a 30-day month with expected precipitation 90 mm, \(q=0.40\), \(\rho=0.30\), \(k=0.8\), and \(r\_0=0.1\) mm:

| Derived quantity | Value |
| --- | --- |
| Rain after a dry day, \(p\_{01}\) | 0.28 |
| Rain after a wet day, \(p\_{11}\) | 0.58 |
| Mean wet-day precipitation | 7.5 mm |
| Gamma scale for the shifted distribution | 9.25 mm |
| Mean dry spell | 3.57 days |
| Mean wet spell | 2.38 days |

These values are calculated from the stated assumptions, not fitted observations.

A first-order chain is a good baseline, but its geometric spell lengths are restrictive. For monsoon breaks, blocked circulation and extended drought, add a **regional regime state** with fitted duration distributions. Suitable states might include fair/dry, widespread frontal precipitation, convective conditions and persistent cold-air intrusion. Fit regime persistence and within-regime wet-day persistence jointly; otherwise the same persistence can be counted twice.

### 1.4 Generate temperature, humidity, radiation and wind jointly

Independent sampling can produce physically inconsistent combinations: persistent heavy rain with clear-sky radiation, or dew points above air temperature.

Use seasonally detrended, regime-conditioned residuals:

\[
\mathbf z\_t=A\mathbf z\_{t-1}+B\boldsymbol\epsilon\_t
\]

where \(\mathbf z\_t\) contains standardized anomalies and \(\boldsymbol\epsilon\_t\) is a vector of independent standard-normal draws. Choose \(A\) to be stable and \(B\) to reproduce the desired innovation covariance.

Joint, precipitation-conditioned temperature generation is a central feature of the Richardson approach. Modern regional generators extend the conditioning to circulation patterns and climate covariates. [AGU Journals](https://agupubs.onlinelibrary.wiley.com/doi/10.1029/WR017i001p00182)

My recommended output construction is:

**Temperature:** generate daily mean and a positive diurnal range, then derive minimum and maximum. A logarithmic range avoids accidentally generating \(T\_{\min}>T\_{\max}\). Apply nighttime terrain corrections separately.

**Humidity:** generate dew point or vapour pressure, enforcing physical bounds. Relative humidity can then be derived from temperature.

**Radiation:** generate cloud attenuation against the astronomical clear-sky/daylight envelope.

**Wind:** generate vector components conditioned on circulation; retain a separate gust variable. Daily mean wind is insufficient for tree damage and rapid fire spread.

For evapotranspiration, pass temperature, radiation, humidity and wind to a consistent formulation such as FAO Penman–Monteith. Do not infer evaporative demand solely from temperature where the other variables are available. [FAOHome](https://www.fao.org/4/x0490e/x0490e06.htm)

### 1.5 Spatial coherence is essential

Each farm should not roll independent regional weather. Conversely, an entire large region should not receive identical rain.

The strongest practical pattern is:

**Shared circulation state + spatially correlated anomalies + local terrain corrections + moving precipitation footprints.**

Multisite generators explicitly preserve spatial and cross-variable dependencies. Nguyen and colleagues’ nsRWG, for example, conditions a regional generator on circulation patterns and temperature, producing spatially and temporally consistent fields. [ASCMO](https://ascmo.copernicus.org/articles/10/195/2024/)

For TCE, use inexpensive correlated fields for ordinary daily anomalies and explicit footprints for important storms. Allow temperature anomalies to be smoother than convective rainfall. Fit correlation lengths by variable and season rather than assigning one universal distance.

For damaging storms, disaggregate daily precipitation into pulses:

\[
P\_t=\sum\_h P\_{t,h}
\]

The pulses must conserve the daily total. Their duration and peak intensity determine whether rainfall infiltrates gently or causes rapid runoff. CLIGEN explicitly generates storm duration, peak intensity and time to peak for erosion modeling, making it a useful reference for this part of the system. [ARS](https://www.ars.usda.gov/midwest-area/west-lafayette-in/national-soil-erosion-research/docs/wepp/cligen/)

### 1.6 Slow variability and drought

An endless simulation needs variability beyond weather persistence. Otherwise decades become suspiciously interchangeable.

A minimal slow climate state is:

\[
s\_{m+1}=a\,s\_m+\sqrt{1-a^2}\,\epsilon\_m
\]

with \(a=\exp(-1/\tau)\), where \(\tau\) is a persistence timescale in months.

Use regional loadings of this state to modify precipitation occurrence, intensity, temperature and storm-regime probabilities. Different regions may respond with opposite signs.

For an Earth analogy, ENSO events recur irregularly at roughly **2–7-year intervals**, commonly last **9–12 months**, and sometimes persist longer. A fictional world need not reproduce the Pacific’s exact teleconnections. Also, an AR(1) process produces red noise, not a preferred oscillation; use a damped AR(2) or another oscillatory process when that distinction matters. [National Ocean Service](https://oceanservice.noaa.gov/facts/ninonina.html)

Steinschneider and Brown demonstrate a more elaborate implementation: low-frequency annual variability conditions a multisite daily weather generator. Their work also highlights the difficulty of inferring robust slow variability from short records. [AGU Journals](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1002/wrcr.20528)

Two numerical precautions matter:

**Preserve the mean.** Multiplying precipitation by \(\exp(\sigma s)\) raises its expectation. For a standard-normal \(s\), \(\exp(\sigma s-\sigma^2/2)\) has mean one. Logistic modifications to occurrence probability need their own mean calibration.

**Partition variance.** Fit slow variability and daily variability together. Adding an arbitrary annual multiplier to an already calibrated generator can make annual rainfall too variable.

Finally, distinguish three drought states:

| Drought type | What TCE should track |
| --- | --- |
| Meteorological | Precipitation deficit relative to the local seasonal distribution |
| Agricultural | Root-zone water deficit and crop-stage-specific stress |
| Hydrological | Streamflow, reservoir, snowpack and groundwater deficits |

The WMO SPI guide explicitly distinguishes the shorter response of soil moisture from the longer memory of groundwater, streamflow and storage. SPI itself contains no soil-water balance or evaporative-demand calculation. [Drought Management](https://www.droughtmanagement.info/literature/WMO_standardized_precipitation_index_user_guide_en_2012.pdf)

A minimal field water balance is:

\[
W\_{t+1}=W\_t+P\_{\mathrm{effective}}+I+M-ET\_{\mathrm{actual}}-D-Q
\]

where \(I\) is irrigation, \(M\) snowmelt, \(D\) drainage and \(Q\) runoff. Account for overflow and unavailable water explicitly rather than silently clipping away mass.

---

## 2. Parameters: quantitative anchors and starter ranges

### 2.1 Evidence-backed quantities

“High confidence” below means confidence in the stated definition or observation—not that the same number transfers to every fictional location.

| Quantity | Value or range | Units | Source and confidence |
| --- | --- | --- | --- |
| Baseline environmental lapse rate | 6.5 | °C/km | Standard-atmosphere/global-average reference; **high as a baseline, lower locally**. NOAA. [NOAA](https://prod-01-alb-www-noaa.woc.noaa.gov/jetstream/appendix/weather-glossary-l?utm_source=chatgpt.com) |
| Dry adiabatic lapse rate | Approximately 9.8 | °C/km | Rising/sinking unsaturated air parcel, **not** a universal terrain temperature correction. **High**. NOAA. [NOAA](https://prod-01-alb-www-noaa.woc.noaa.gov/jetstream/appendix/weather-glossary-l?utm_source=chatgpt.com) |
| Singapore annual precipitation, 1991–2020 | 2,113.3 | mm/year | Changi climate station; **high, site-specific**. [Meteorological Service Singapore](https://www.weather.gov.sg/climate-climate-of-singapore/) |
| Singapore rain days | 171, using ≥0.2 mm/day | days/year | Shows the importance of the threshold; **high, site-specific**. [Meteorological Service Singapore](https://www.weather.gov.sg/climate-climate-of-singapore/) |
| Darwin January versus July precipitation | 432.1 versus 1.1 | mm/month | BOM all-years station statistics; **high for the displayed record, not a common-period global comparison**. [Bureau of Meteorology](https://www.bom.gov.au/climate/averages/tables/cw_014015.shtml) |
| Darwin January versus July wet days | 19.0 versus 0.2, using ≥1 mm/day | days/month | Corresponding wet-day fractions approximately 0.613 and 0.0065; **site-specific**, calculated fractions. [Bureau of Meteorology](https://www.bom.gov.au/climate/averages/tables/cw_014015.shtml) |
| Perth July versus December precipitation | 147.0 versus 9.6 | mm/month | Southern Hemisphere winter-wet example; **high, record-specific**. [Bureau of Meteorology](https://www.bom.gov.au/climate/averages/tables/cw_009225.shtml) |
| Tokyo January versus August mean temperature | 5.4 versus 26.9 | °C | JMA 1991–2020 normals; **high, site-specific**. [Japan Meteorological Agency](https://www.data.jma.go.jp/stats/etrn/view/nml_sfc_ym.php?block_no=47662&day=&month=&prec_no=44&view=p1&year=) |
| Published precipitation-tail examples | Fort Collins: threshold 10 mm/day, shape \(\xi\approx0.18\); Pergamino, Argentina: threshold 16 mm/day, fitted \(\xi\) roughly 0.05–0.10 | mm/day; dimensionless shape | Furrer and Katz; **medium for transfer**, not global constants. [DOI](https://doi.org/10.1029/2008wr007316?utm_source=chatgpt.com) |

These observations immediately rule out several tempting shortcuts. Tropical climates need not have rain every day. A place can receive substantial annual rain while having an almost rainless season. Similar annual totals need not imply similar wet-day frequency or storm intensity.

### 2.2 Proposed parameter families by climate type

**The following are deliberately broad TCE initialization ranges, not measured Köppen-wide distributions.** Their numerical confidence is **low until calibrated**. They provide a search space for selecting analogues and constructing prototype worlds.

Here, \(q\) is the fraction of days with at least **1 mm of liquid-equivalent precipitation**, and \(\sigma\_T\) is the standard deviation of daily mean temperature anomalies **after removing the seasonal cycle**.

| Climate family | Prototype annual precipitation | Wet-season \(q\) | Dry-season \(q\) | Prototype \(\sigma\_T\) | Essential behavior |
| --- | --- | --- | --- | --- | --- |
| Af: ever-wet tropical | 1,800–3,500 mm | 0.45–0.70 | 0.30–0.50 | 0.7–1.8°C | Rain throughout the year; strong within-day variability |
| Am/Aw: monsoon or savanna | 800–2,500 mm | 0.45–0.80 | 0.00–0.15 | 1–3°C | Onset, withdrawal and intraseasonal breaks |
| BWh/BWk: desert | 30–250 mm | 0.03–0.15 | 0.00–0.03 | 2–6°C | Many dry months; occasional important storms |
| BSh/BSk: steppe | 250–650 mm | 0.15–0.40 | 0.01–0.10 | 2–5°C | Large agricultural consequences from rainfall timing |
| Csa/Csb: dry-summer temperate | 350–1,000 mm | 0.30–0.60 in winter | 0.01–0.12 in summer | 2–4°C | Winter recharge; summer soil and fuel drying |
| Cfa/Cwa: warm humid or winter-dry | 800–2,000 mm | 0.30–0.65 | 0.08–0.35 | 2–5°C | Warm-season heavy rain; region-dependent winter dryness |
| Cfb/Cfc: maritime temperate | 600–1,800 mm | 0.30–0.60 | 0.20–0.45 | 2–4°C | Frontal sequences; moderate temperature seasonality |
| Dfa/Dfb/Dwa/Dwb: continental | 350–1,200 mm | 0.25–0.55 | 0.08–0.35 | 3–6°C | Strong annual cycle; frost, snow and cold-air episodes |
| Dfc and related subarctic | 200–800 mm | 0.20–0.45 | 0.08–0.25 | 3–7°C | Short growing season; snow-storage memory |
| ET: tundra | 100–600 mm | 0.15–0.40 | 0.05–0.25 | 2–6°C | Summer warmth limited; freeze–thaw and snow timing |

Do not draw all columns independently. Build twelve monthly profiles, derive positive-day amounts from monthly precipitation and occurrence, and then run the Köppen classifier. Some combinations within these broad search ranges will belong to a different class.

Temperature seasonality must likewise be fitted separately from daily variability. A continental climate with \(\sigma\_T=5^\circ\mathrm C\) does not have a five-degree annual temperature range.

For remaining stochastic parameters, these are reasonable **engineering starting points only**:

| Parameter | Proposed starting range | Calibration target |
| --- | --- | --- |
| Within-regime wet/dry persistence \(\rho\) | 0.1–0.5 | Wet/dry spell distributions |
| Gamma shape \(k\) for ordinary wet-day amounts | 0.5–2.0 | Positive precipitation mean, variance and quantiles |
| Daily temperature anomaly AR coefficient | 0.4–0.8 | Detrended lag-one correlation |
| Persistent circulation episode duration | Initially explore 3–10 days | Observed regime-duration distribution; allow longer tails |
| Slow-state persistence timescale \(\tau\) | Initially explore 6–36 months | Annual variance and multi-year dry/wet runs |

These defaults should not survive unchanged merely because the generated weather looks plausible on screen.

### 2.3 Drought and cold-snap thresholds

For SPI, the standardized normal distribution supplies a useful comparison scale:

| Threshold | Interpretation | Idealized fraction of values at or below threshold |
| --- | --- | --- |
| SPI ≤ −1 | Moderate-or-worse precipitation deficit | 15.9% |
| SPI ≤ −1.5 | Severe-or-worse deficit | 6.7% |
| SPI ≤ −2 | Extreme deficit | 2.3% |

The categories follow WMO; the percentages are normal-distribution calculations. **They are not annual probabilities of new drought events.** Rolling windows overlap, consecutive values are correlated, and zero-heavy dry-season precipitation requires care. Use SPI for diagnostics, not as a direct crop-yield multiplier. [Drought Management](https://www.droughtmanagement.info/literature/WMO_standardized_precipitation_index_user_guide_en_2012.pdf)

For cold snaps, retain both an absolute and a relative measure. Frost exposure depends on actual temperatures and crop development; climatological unusualness can use the ETCCDI cold-spell definition: **at least six consecutive days with daily minimum temperature below the calendar-day 10th percentile**. Its heat-spell counterpart uses daily maxima above the 90th percentile. [Climdex](https://www.climdex.org/learn/indices/)

### 2.4 Storm magnitude and frequency

A gamma-only amount distribution may underrepresent heavy precipitation. Furrer and Katz found that a **gamma body with a generalized Pareto upper tail** could substantially improve extreme-event simulation, while also documenting remaining limitations. [AGU Journals](https://agupubs.onlinelibrary.wiley.com/doi/abs/10.1029/2008WR007316?utm_source=chatgpt.com)

For excess \(Y=P-u\) above threshold \(u\):

\[
\Pr(Y>y)=
\left(1+\xi\frac{y}{\beta}\right)^{-1/\xi}
\]

within its valid support. Fit the threshold, tail scale and shape; do not transfer the Colorado or Argentine estimates indiscriminately.

Track storm episodes separately from exceedance days when events cluster. A three-day storm is not three independent regional disasters. Also, storm rainfall must be part of the precipitation model—not an extra amount added on top of an already complete rainfall distribution.

For a stationary annual exceedance probability \(1/R\), the probability of at least one exceedance in \(N\) independent years is:

\[
1-(1-1/R)^N
\]

Thus a “100-year” event has a **63.4%** chance of occurring at least once in 100 years. It should not be scheduled once per century.

Treat tropical cyclones as a separate track-and-footprint process. Calibrate basin seasonality, tracks and intensity from **IBTrACS**, which combines best-track records from multiple forecasting agencies. A Köppen label alone does not determine cyclone exposure. [NCEI](https://www.ncei.noaa.gov/products/international-best-track-archive)

---

## 3. Variation across eras and world regions

### 3.1 Climate should not advance with technology

TCE should have separate **environmental history** and **technological history**. Foragers and farmers can experience the same climate. Steam power should not automatically trigger an Earthlike warming trajectory.

| Social/technological context | Environmental representation | Simulation implication |
| --- | --- | --- |
| Foragers | Choose an actual climate background: glacial, interglacial or another defined state. | Do not equate “forager” with cold climate. Mobility and ecological knowledge affect exposure. |
| Early farming | Retain geographical and orbital forcing; allow regionally different seasonal climates. | Crop calendars and domesticated species should adapt to local growing conditions. |
| Pre-industrial | Include internal variability and, where desired, external natural forcing such as volcanic episodes. | Bad decades should be possible without industrial emissions. |
| Industrial | Couple climate change to specified global emissions and land-use forcing, not an era flag. | A small industrial settlement should not cause planetary warming by itself. |
| Modern | Use an explicitly dated baseline and nonstationary parameters when appropriate. | Historical observations cannot all be treated as samples of one unchanged distribution. |

There are important numerical anchors:

**Glacial background.** Tierney and colleagues reconstructed Last Glacial Maximum global cooling of approximately **6.1°C**, with a 95% interval of **5.7–6.5°C**, relative to the pre-industrial state. This is a global mean—not a uniform amount to subtract from every location and month. [Nature](https://www.nature.com/articles/s41586-020-2617-x?utm_source=chatgpt.com)

**Early-to-middle Holocene backgrounds.** PMIP4 mid-Holocene experiments represent approximately **6,000 years ago**, with different seasonal and latitudinal insolation. Simulations show expanded monsoon influence, but generally less expansion than indicated by palaeoclimate reconstructions. This is an important area of model–proxy disagreement, particularly for northern Africa. [Copernicus Publications](https://cp.copernicus.org/articles/16/1847/2020/)

**Modern comparison.** IPCC AR6 assessed **2011–2020** global surface temperature as **1.09°C [0.95–1.20°C]** above 1850–1900, with larger warming over land than ocean. That is a dated calibration reference, not a claim about the latest annual temperature. It also shows why globally subtracting one number from modern station records does not reconstruct pre-industrial local climate. [IPCC](https://www.ipcc.ch/report/ar6/wg1/chapter/summary-for-policymakers/?utm_source=chatgpt.com)

For a centuries-long early-agrarian TCE run, my default would be a stationary background climate plus realistic internal variability. Add slow externally forced change only when the world’s environmental specification calls for it.

### 3.2 Worldwide variation that matters to the generator

**Equatorial Southeast Asia is not “constant weather.”** Singapore has a small annual temperature cycle but strong daily heating, two monsoon seasons, squalls and convective rain. Its wet-day frequency and precipitation intensity still vary seasonally. [Meteorological Service Singapore](https://www.weather.gov.sg/climate-climate-of-singapore/)

**Tropical northern Australia needs an extreme wet–dry contrast.** Darwin’s January and July observations show why a generic tropical rain probability is inadequate. The dry season is normal climate, not automatically a drought. [Bureau of Meteorology](https://www.bom.gov.au/climate/averages/tables/cw_014015.shtml)

**Southwestern Australia illustrates Southern Hemisphere winter rainfall.** Perth’s wettest season occurs when solar energy and evaporative demand are lower, unlike a summer-monsoon regime. TCE should allow the same annual precipitation total to produce very different seasonal water availability. [Bureau of Meteorology](https://www.bom.gov.au/climate/averages/tables/cw_009225.shtml)

**East Asian humid climates combine substantial temperature seasonality with complex rainfall seasonality.** Tokyo’s 1991–2020 normals run from 5.4°C in January to 26.9°C in August; October precipitation, 234.8 mm, exceeds January’s 59.7 mm. A single rainfall sine wave tied to temperature would miss that structure. [Japan Meteorological Agency](https://www.data.jma.go.jp/stats/etrn/view/nml_sfc_ym.php?block_no=47662&day=&month=&prec_no=44&view=p1&year=)

**Monsoon Asia, Africa and northeastern Brazil can suffer related multi-year droughts without identical local weather.** Singh and colleagues’ analysis of 1875–1878 links concurrent droughts to an exceptional combination of Pacific, Indian and Atlantic conditions. In each of 1876–1878, at least a quarter of Monsoon Asia was affected by drought in their reconstruction. This is a strong argument for shared slow climate drivers with regional responses. [AMS Journals](https://journals.ametsoc.org/view/journals/clim/31/23/jcli-d-18-0159.1.xml?utm_source=chatgpt.com)

**South American precipitation tails need regional calibration.** The Pergamino and Fort Collins results above demonstrate that even the heaviness of extreme-rainfall tails differs between sites; “continental agricultural climate” is not a sufficient parameter specification. [DOI](https://doi.org/10.1029/2008wr007316?utm_source=chatgpt.com)

---

## 4. Stylized facts and validation targets

A correct generator should reproduce distributions and dependencies, not merely attractive averages.

| Pattern to reproduce | Quantitative or operational test |
| --- | --- |
| Seasonal means remain correct without forced totals | Compare generated monthly means against targets across long ensembles; individual years must vary. |
| Rain clusters into spells | Match wet-day frequency, transition probabilities, and median/upper-quantile wet and dry spell lengths. |
| Daily and annual temperature variability differ | Fit the seasonal cycle first, then test anomaly standard deviation and autocorrelation. |
| Monsoons can fail through timing | Measure onset, withdrawal, longest growing-season break and rainfall during crop-critical stages. |
| Extremes have realistic tails | Compare annual maximum 1-day and 5-day precipitation, and estimated 10-, 20- and 100-year levels with uncertainty. |
| Drought has several timescales | Compare 1-, 3-, 6-, 12- and 24-month precipitation deficits, plus soil and storage deficits. |
| Nearby places share hazards imperfectly | Test precipitation correlation versus distance and the fraction of agricultural area simultaneously dry. |
| Cold snaps are sustained and seasonal | Compare frost dates, frost-day counts and six-day cold-spell statistics. |
| Water and energy inputs remain physical | No negative precipitation, impossible humidity, reversed minimum/maximum temperatures or radiation during polar night. |
| Rare events are irregular | Test exceedance counts and waiting times, not predetermined disaster spacing. |

The precipitation and temperature extreme indices are aligned with the ETCCDI/Climdex definitions, including annual maximum one-day and five-day rainfall and consecutive dry days below 1 mm/day. [Climdex](https://www.climdex.org/learn/indices/)

**Recommended validation procedure:** fit against observed daily records, withhold some years or stations, and generate ensembles of independent record-length simulations. Compare observed statistics to ensemble distributions.

For offline calibration, a few thousand synthetic years per archetype are practical as a testing choice, but they do not create information absent from the observations. Under an independent stationary model, a 30-year record has about a **74% probability of containing no exceedance of a true 100-year threshold**. Tail estimates therefore need regional pooling and uncertainty bounds.

Also test the systems driven by weather. A generator can reproduce daily precipitation reasonably while getting basin flood peaks or crop-failure synchrony wrong because spatial dependence or event duration is wrong. Multisite weather-generator studies explicitly evaluate those dependencies, and some still show residual biases in persistence and extremes. [AGU Journals](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1002/wrcr.20528)

---

## 5. Modeling recommendation for TCE

### 5.1 Recommended architecture and update rates

I would use four layers:

| Layer | Proposed representation | Update cadence |
| --- | --- | --- |
| Background climate | Twelve monthly fields: temperature, precipitation, circulation, humidity/radiation statistics | World creation; update only for specified climate change |
| Slow regional climate | A small vector of persistent ocean/circulation anomalies | Monthly |
| Regional weather | Circulation regime, correlated anomaly fields, storm objects | Daily; storm motion subdaily |
| Local surface state | Soil water, snow, groundwater inputs, fuel moisture and terrain-adjusted temperature | Daily, with shorter event steps where necessary |

For a settlement-sized playable map, start with a coarse weather lattice—perhaps **1–4 km cells as an engineering choice**—plus terrain downscaling and storm footprints. Do not run atmospheric calculations on the terrain’s finest heightfield.

A minimum daily weather record should contain:

```
WeatherDay {
    minimum_temperature_c
    maximum_temperature_c
    precipitation_water_equivalent_mm
    snowfall_water_equivalent_mm
    solar_radiation_mj_m2
    vapour_pressure_kpa
    mean_wind_vector_m_s
    maximum_gust_m_s
    circulation_regime_id
    storm_event_ids
}
```

Keep optional intraday precipitation pulses for hydrology and erosion. Snowfall is a partition of total precipitation, not an additional input.

The update sequence should be:

```
advance slow climate state when the month changes
advance regional circulation regimes
generate spatially correlated precipitation and temperature anomalies
sample precipitation occurrence and amount
generate jointly consistent humidity, radiation and wind
apply local terrain and precipitation-phase corrections
disaggregate important storm events without changing their totals
update snow, soil, fuel moisture and hydrological inputs
publish one authoritative weather result to agents and rendering
```

### 5.2 Individual agents and institutions

**Weather belongs to places; decisions belong to agents.**

A farmer should observe recent rain, soil conditions, seasonal cues and communicated information—not access the generator’s future random draws. Households can differ in knowledge, risk tolerance, seed reserves and access to irrigation while experiencing the same physical weather.

I would connect the weather system to institutions through explicit resources and actions:

| System | Weather-driven state | Agent or institutional response |
| --- | --- | --- |
| Farming | Root-zone water, frost exposure, accumulated warmth, waterlogging | Planting date, crop choice, labour allocation, irrigation |
| Food security | Spatially correlated harvest outcomes | Storage, imports, rationing, lending, relief |
| Water management | Reservoir inflow, low flows, snowmelt timing | Allocation rules, maintenance, investment and conflict |
| Fire | Fuel moisture, wind, gusts and ignition conditions | Burning practices, suppression, evacuation, construction choices |
| Disease | Pathogen-specific environmental conditions | Water handling, sanitation, shelter, treatment and movement |

These are recommended interfaces, not universal historical response laws.

Avoid a direct rule such as `drought → famine → revolt`. Mishra and colleagues’ historical analysis of India distinguishes droughts associated with famines from major droughts that did not produce famine. TCE should likewise let weather affect production and access while institutions, distribution and household reserves determine consequences. [AGU Journals](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2018GL081477?utm_source=chatgpt.com)

### 5.3 Existing models worth borrowing from

| Model | What to borrow | What not to assume |
| --- | --- | --- |
| **Richardson/WGEN family** | Wet/dry persistence and precipitation-conditioned multivariate weather | Ordinary daily calibration automatically reproduces multi-year drought |
| **GWGEN 1.0** | Lightweight conversion of monthly inputs to daily minimum/maximum temperature, precipitation, cloud and wind | Published v1.0 does **not** provide spatially autocorrelated multipoint weather |
| **Steinschneider–Brown generator** | Coupling low-frequency variability to multisite daily weather | A short local record identifies decadal variability reliably |
| **nsRWG** | Circulation-conditioned, spatially coherent fields and nonstationary climate covariates | Its Central European calibration transfers unchanged worldwide |
| **CLIGEN** | Storm duration, peak intensity and time-to-peak | All generated variables have realistic daily dependence |
| **LFPM** | Cheap moisture transport and rain-shadow generation | Orographic precipitation alone supplies a complete global circulation model |

Sources: Richardson; Sommer and Kaplan; Steinschneider and Brown; Nguyen and colleagues; USDA ARS; Hergarten and Robl. [AGU Journals](https://agupubs.onlinelibrary.wiley.com/doi/10.1029/WR017i001p00182)

CLIGEN’s documentation is especially instructive: it warns that several variables are generated independently of precipitation, which can be inappropriate for models sensitive to their daily interactions. Borrow its storm detail, but preserve TCE’s joint weather consistency. [ARS](https://www.ars.usda.gov/midwest-area/west-lafayette-in/national-soil-erosion-research/docs/wepp/cligen/)

### 5.4 Determinism and time acceleration

Keep weather authoritative in the Rust kernel. Unreal should render that weather, not generate a separate physically meaningful sequence.

Use random streams keyed by world, region, simulated date and variable or event identity. Save persistent state: circulation regime, slow modes, previous wet/dry status, autoregressive residuals and active storms.

At high speed, process the same daily weather and surface balances without rendering every intermediate event. At low speed, reveal the already determined intraday realization. Changing camera position, frame rate or simulation speed must not change the next harvest’s rainfall.

The main simplification should be **less atmospheric detail**, not different climatic consequences at different speeds.

---

## 6. Sources, datasets and evidence limits

### Recommended calibration data

| Dataset | Best use | Main limitation |
| --- | --- | --- |
| **GHCN-Daily** | Station precipitation and temperature; occurrence, spells, frost and tails | Uneven coverage and record quality; the full dataset is not homogenized for every observational change. [NCEI](https://www.ncei.noaa.gov/products/land-based-station/global-historical-climatology-network-daily) |
| **WorldClim 2.1** | Approximately 1-km monthly climate surfaces for 1970–2000; geographical analogue selection | Monthly normals cannot identify daily persistence or year-to-year variance. [WorldClim](https://www.worldclim.org/data/worldclim21.html) |
| **ERA5** | Hourly circulation, temperature, humidity, wind and spatial dependence; 1940 onward | Reanalysis, not direct truth; precipitation and extremes require observational checks. [Climate Data Store](https://cds.climate.copernicus.eu/datasets/reanalysis-era5-single-levels?tab=overview) |
| **ERA5-Land** | Finer land-surface context, snow and water-state comparisons; approximately 9-km resolution | Still model-derived; not an independent local weather observation. [Copernicus Climate Change Service](https://climate.copernicus.eu/climate-reanalysis) |
| **CHIRPS v2** | Long precipitation records across much of the tropics and subtropics; useful for agricultural regions | Pin the product version, domain and time period; validate local daily behavior against gauges. [Climate Hazards Center](https://www.chc.ucsb.edu/data/chirps) |
| **IBTrACS** | Tropical-cyclone tracks, seasonality and intensity | Historical completeness and agency conventions must be handled. [NCEI](https://www.ncei.noaa.gov/products/international-best-track-archive) |
| **PMIP and palaeoclimate reconstructions** | Alternative background climates and long-timescale plausibility | Much weaker constraints on exact local daily weather, especially precipitation tails. [Copernicus Publications](https://cp.copernicus.org/articles/16/1847/2020/) |

For implementation, the highest-value scholarly starting points are **Richardson (1981)** for the basic daily generator, **Furrer and Katz (2008)** for precipitation extremes, **Steinschneider and Brown (2013)** for low-frequency variability, **Sommer and Kaplan (2017)** for globally parameterized daily generation, **Hergarten and Robl (2022)** for rain shadows, and **Nguyen et al. (2024)** for spatially coherent circulation-conditioned weather. Their specific contributions and limitations are identified above.

### Where the evidence is strongest—and weakest

**Strong:** seasonal solar geometry; broad circulation mechanisms; the need for joint weather variables; station-level climate normals; the importance of spatial dependence and persistent water storage.

**Moderate and location-dependent:** weather-generator form, regime definitions, precipitation-tail parameters, climate-analogue transfer and the spatial scale of correlation.

**Thin or strongly uncertain:** universal stochastic parameter sets indexed only by Köppen class; precise local pre-instrumental storm return periods; arbitrary-world ocean teleconnections; and confident century-scale tail estimates derived from a few decades of observations. The PMIP monsoon mismatch and documented weather-generator tail biases are concrete examples of these limits. [Copernicus Publications](https://cp.copernicus.org/articles/16/1847/2020/)

**The first version to build:** geography-derived monthly climate, persistent regional regimes, a spatial WGEN-style daily generator, explicit soil/snow/storage memory, and carefully calibrated extreme rainfall. Add complicated planetary climate dynamics only after that system reproduces seasonal timing, regional crop-failure synchrony and multi-year drought. Those are the features that make weather consequential for an emergent civilization—not merely convincing in the sky.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927e8-4b94-83ea-9a77-dacf294d587a)
