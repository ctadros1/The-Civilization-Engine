# Enteric disease between people who work or keep company together: hands, tools and shared food, for The Civilization Engine

## Executive conclusion

**The figures that exist are for households and for common sources, not for people who only work beside each other.** This short note was written for one gap (plan §7, backlog E, the last route of "shedding where people work": exposure of those who share a workplace or a home's company through hands, shared tools and shared food). It reads household secondary attack rates for cholera, typhoid and shigellosis, and the reports of workplace and communal outbreaks, briefly (plan §1: plausible, not proven).

1. **Household contacts are measured, and their rates are high.**
   - Cholera: 22 % of household contacts infected within 21 days in Dhaka.
   - Shigella: 7 % to 40 % across settings, and 26 % of families of children in day care.
   - Typhoid: no clean household figure was found in this session.
2. **Most of that household rate may be shared food and water, not contact between people.** A transmission model of Dhaka households estimated the direct household risk at about 5 % per index case, against the 15 to 22 % of contacts found infected. The rest it put on shared sources. The Dhaka authors also say their findings "likely do not apply to more casual or transient contacts".
3. **Workplace and communal outbreaks of these diseases are reported as common-source events.** Examples are a carrier handling food for a luncheon, a sweet shop's food handler, a break room's food, and a workplace's drinking water. The reports found little or no person-to-person spread among co-workers.
4. **No figure was found for co-worker or co-visitor transmission by hands or shared tools among adults.**

**Conclusion for TCE:** no grounded figure supports a co-worker or co-visitor contact hazard. What the evidence does support is a **food route**: a shedder who prepares food others eat. The kernel cannot yet carry that route, because food batches carry no identity (ADR-0021's "Revisit when": "Goods carry batch identity, making food a route"). Nothing was built from this note.

Throughout, **measured** findings are kept apart from **estimates**. Several sources were reached only through search-engine summaries in this session. They are marked (*search summary*), and none is load-bearing.

---

## 1. Household secondary attack rates

### 1.1 Cholera

**Measured.**
- In Dhaka, household contacts of hospitalized cholera patients were followed for 21 days ([Weil et al. 2009, "Clinical outcomes in household contacts of patients with cholera in Bangladesh", *Clin Infect Dis* 49: 1473–1479, doi:10.1086/644779](https://pmc.ncbi.nlm.nih.gov/articles/PMC2783773/); abstract and summary read).
  - Contacts were people "sharing the same cooking pot for at least three days".
  - 202 of 944 (22 %) had a positive rectal swab; 148 of the 202 infected (73 %) had diarrhoea.
  - Infections clustered early: symptoms most often began two days after the index case presented.
  - The authors attribute multiple infections in a household to "shared contaminated food and water" or "fecal-oral transmission within households", and did not separate the two.
  - They state that the findings "likely do not apply to more casual or transient contacts".
- Two cohorts from the same group: 78 of 506 contacts (15 %) were infected (*search summary*).

**Estimate.**
- A transmission model of Dhaka households, 2001–2006, put the risk of infection among household contacts through direct exposure to a shedder at **4.9 %** (95 % CI 0.9–22.8 %) over an 11-day infectious period (*search summary*). The rest of the household excess it attributed to shared sources.

### 1.2 Shigellosis (bacillary dysentery)

**Measured** (all *search summary*, except where a figure is a range compiled by a health department):
- **Families of day-care children:** in a 1981 prospective study of 20 day-care centres, the secondary attack rate of diarrhoea in their families was 26 % for Shigella (15 % rotavirus, 17 % Giardia).
- **Amsterdam, 2002–2009:** infection among high-risk household contacts of primary cases was 7.4 %.
- **Compiled ranges:** household secondary attack rates of 10 to 40 % (Louisiana and North Carolina health department summaries), and bacillary dysentery in Saudi family clusters from 7.7 to 80 %.
- **Infectious dose:** Shigella's small infectious dose makes it "more likely to spread person-to-person than other enteric pathogens", for as long as organisms are in the faeces (1–4 weeks).

### 1.3 Typhoid

**No household secondary attack rate** was found in this session. Typhoid's outbreaks are reported by source: water, and food prepared by a carrier (§2).

---

## 2. Workplace and communal outbreaks

**Measured.**
- **Michigan, 1981:** a typhoid carrier, an asymptomatic 68-year-old woman who helped prepare most of the food at a luncheon, infected 18 of 310 volunteers (an attack rate of 5.8 %). The investigators found no evidence of secondary transmission ([CDC MMWR](https://www.cdc.gov/mmwr/preview/mmwrhtml/00001174.htm); *search summary*).
- **South Dumdum, West Bengal, 2007:** a sweet shop's food handler kept serving while ill with typhoid and did not wash his hands. Eating its milk products was associated with illness (matched OR 6.2). A second, waterborne wave followed from piped water contaminated by an open drain (matched OR 7.3). The overall attack rate was 74 per 10,000 ([Bhunia et al. 2009, *BMC Public Health* 9: 115, doi:10.1186/1471-2458-9-115](https://pmc.ncbi.nlm.nih.gov/articles/PMC2683821/); read). The paper does not analyse spread within households.
- **Shigella at workplaces:**
  - A Texas laboratory outbreak followed food left in a break room.
  - A Belgian cafeteria outbreak was traced to a food handler back from travel, without symptoms.
  - A Flemish workplace outbreak of *S. sonnei* had an attack rate of 14 % among respondents. Its investigation looked to the workplace's drinking water from new cooling machines, and the source was not confirmed.
  - All of these are *search summary*.

**Estimate (spatial, not a contact rate).** In Bangladesh, Chad and the Democratic Republic of the Congo, cholera risk is raised for those living within tens to a few hundred metres of a case in the days after it presents. For example, 1.9 and 3.9 times the city's risk at 75–125 m within five days (*search summary*). This is neighbourhood risk by any route, mostly shared water. It is not a measure of contact between people who work together.

---

## 3. What the rate would depend on

From the sources above (qualitative):
- **Shared food and water dominate.** A shared cooking pot defines the cholera household, and a food handler is the workplace's common source.
- **Shigella's small dose makes hands matter more for it than for cholera.** Children in day care carry it home.
- **Duration and intimacy of contact.** The Dhaka authors exclude "casual or transient contacts" from their household figure.

---

## 4. TCE rules (proposed)

- **No co-worker or co-visitor contact hazard is built.** *Reason:* no measured or estimated rate exists for adults who work or keep company together, and the household rate the kernel uses (`household_hazard`, a tuning value) is, by the best estimate here, mostly shared food and water. A contact hazard scaled from it would count the same shared sources a second time. 05-03 §1 also warns: "Do not treat every person within an outdoor radius as equivalent to a household member".
- **What would be grounded later is a food route.** A shedder who prepares food others eat (a household's cook, a firm's or a sickhouse's kitchen, a seller of prepared food) would pass a load to what they prepared, and those who eat it would take a dose. Section 2's outbreaks are of this kind. It needs food batches with identity (ADR-0021's "Revisit when": "Goods carry batch identity, making food a route").
