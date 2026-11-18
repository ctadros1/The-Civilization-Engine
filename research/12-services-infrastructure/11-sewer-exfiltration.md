# What leaks out of a worn sewer: exfiltration, how far it reaches and what it depends on, for The Civilization Engine

## Executive conclusion

**A sewer leaks a few per cent of what it carries, more as its walls wear, and what leaks out stays close to the pipe.** This short note was written for one gap (plan §7, backlog C's "Not yet": leakage through worn walls, deferred for "a rate the research lacks"). It reads the field measurements of exfiltration from sewers and what is known of where the leaked sewage goes, briefly (plan §1: plausible, not proven). Its findings:

1. **Measured exfiltration is about 1 to 10 per cent of the flow.** Reviews of modern systems put it at 1 to 5 per cent of dry-weather flow, with a median of about 2.4 per cent. A tracer study of a 2 km reach of unknown condition measured 9.9 ± 2.7 per cent. A water balance of a Rastatt suburb's sewers, about 25 years old and partly damaged, gives about 4 per cent of household wastewater.
2. **It comes from a minority of the pipe, its defects.** Most of a system's exfiltration comes from 20 to 50 per cent of its length. Rehabilitation cuts it by 50 to 90 per cent. A new pipe meets a construction standard for allowable leakage, which is small.
3. **Per defect, it is linear in the leak's area and the head over it, and a leak clogs.** Laboratory and field work finds exfiltration through a gap falls quickly to a steady state as a clogging (colmation) layer forms, and rises again in storms when the head rises or the layer is disturbed.
4. **A pipe below the water table takes water in instead of letting sewage out.** Where groundwater stands above the pipe, the flow runs into it (infiltration), and in wet ground infiltration is much larger than exfiltration.
5. **What leaks reaches only a narrow zone around the pipe.** Most of it is broken down and attenuated in the unsaturated ground just below the leak. Modelled virus transport in an aquifer reaches 0.2 to 14 m. Regulations keep wells 15 m (50 ft) from a sewer, or 6 m (20 ft) for a tested tight pipe. Microbes from sewers have nonetheless been found deep in a fissured sandstone aquifer.

Throughout, **measured** findings are kept apart from **estimates** and **proposed TCE rules**. Several sources were reached only through search-engine summaries in this session. They are marked (*search summary*), and none is load-bearing without a design prior's label.

---

## 1. How much leaks out

### 1.1 As a share of the flow

**Measured and compiled.**
- The Chesapeake Bay Program's 2025 review of sewer exfiltration compiles studies that state exfiltration as a share of the flow ([Delesantro, CBP 2025](https://www.chesapeakebay.net/files/documents/Sanitary-Sewer-Exfiltration_July25_JDel_072325.pdf), slides; read in full). The shares, with the review's labels for where each was observed:

  | Study | Share | Observed | Basis |
  |---|---|---|---|
  | Nguyen and Venohr 2021 | 2 % | to groundwater | dry-weather flow |
  | Delesantro et al. 2022 | 2.4 % | to stream | dry-weather flow |
  | Steele et al. 2025 | 0.6 % | from pipe | total flow |
  | Amik et al. 2000 | 11.4 % | from pipe | dry-weather flow (an outlier) |
  | Ellis et al. 2004 | 3–5 % | from pipe | review |
  | Fenz 2003 | 1–5 % | from pipe | dry-weather flow |
  | Fenz et al. 2005 | 1 % | to groundwater | dry-weather flow |
  | Karpf and Krebs 2004 | 2.8 % | to groundwater | dry-weather flow |
  | Giulianelli et al. 2003 | 0.24–2.96 % | from pipe | dry-weather flow |
  | Yang et al. 1999 | 1–2 % | to groundwater | total flow |
  | CIRIA 1995 | 3 % | from pipe | total flow, leaky system |

  The review took the median of the field values, 2.4 per cent of dry-weather flow, as its default. It also gives a wider literature range of 3–48 per cent from other compilations, and notes that laboratory analyses tend to run higher.
- A tracer method on a 2 km reach "of unknown condition" measured exfiltration at **9.9 ± 2.7 per cent** of the flow ([Rieckermann et al. 2005, *Water Resources Research*, doi:10.1029/2004WR003699](https://borsuk.pratt.duke.edu/publications/novel-tracer-method-estimating-sewer-exfiltration); authors from a *search summary*). The uncertainty comes mostly from using salt as the tracer.
- A UK test rig measured 0.1 to 1 per cent of the pipe's flow in most cases, and 1 to 3 per cent at radial defects (*search summary* of a rig study cited in the review literature).

### 1.2 In a worn sewer, by water balance

**Measured, with an estimate derived from the reported figures.** In Plittersdorf, a suburb of Rastatt in the Upper Rhine valley, the sewers were built mainly in 1965–1970 and studied in 1993–1995. The network had 9,955 m of foul sewer, with a mean diameter of 250 mm ([Eiswirth and Hötzl 1997, "The impact of leaking sewers on urban groundwater", in Chilton et al. (eds), *Groundwater in the urban environment*, pp. 399–404](https://users.pfw.edu/isiorho/G300Eiswirth-Hoetzl_IAH_Nottingham_1997.pdf); read in full). Its findings:
- Exfiltration of **1,200 L a day per km** of sewer, against groundwater infiltration of 46,600 L a day per km. In Hannover the figures were 17,300 and 198,720 L a day per km (Härig and Mull 1992, as cited).
- That comes to 4,400 m³ of sewage a year leaking from the suburb's sewers, against a mean water demand of 320 m³ a day. This is **about 3.8 per cent of the household wastewater** (an estimate: 4,400 / (320 × 365)).
- About 87 per cent of the system lay within the zone of a fluctuating water table, so whether a defect let sewage out or groundwater in depended on the level.

### 1.3 Where in the pipe, and with age

**Compiled; the shares are the review's central values for expert judgement.**
- Most exfiltration comes from a fraction of the system, 20–50 per cent of its length. Rehabilitation reduces it by 50–90 per cent. A pipe is treated as new or newly rehabilitated for 10 years. Force mains, which are designed to be watertight, are excluded from leakage ([CBP 2025](https://www.chesapeakebay.net/files/documents/Sanitary-Sewer-Exfiltration_July25_JDel_072325.pdf)).
- Sewers are not designed to be watertight. The US construction standard allows about 125 gallons per 400 ft of pipe, or about 1,650 gallons per mile (ASTM 2009, as cited by the Chesapeake Bay Program's 2014 expert panel report in the same slides).

---

## 2. What the rate depends on

**Measured.**
- **The leak's area and the head over it.** Rauch and Stegner (1994) found the flow through a leak to be linear in its area and the pressure head. They found its permeability to fall with time as the bedding sand around it clogged (*search summary*).
- **Clogging of the leak.** Sewer rig studies show exfiltration from gaps and joints falling exponentially to a steady state of 10⁻³ to 10⁻⁶ L/s. The clogging layer at the pipe's base is disturbed by changes in head and flow ([Ellis, Revitt, Vollertsen and Blackwood 2009, "Sewer exfiltration and the colmation layer"](https://repository.mdx.ac.uk/item/81wz0); *search summary*).
- **Storms.** Vollertsen and Hvitved-Jacobsen (2002) found about 0.06 L a day per cm² of leak once a clogging layer had nearly sealed it. Simulated storm events raised leakage up to 56 times above the sealed rate (*search summary* of a study citing them).
- **The water table.** A pipe below the water table infiltrates rather than exfiltrates ([Eiswirth and Hötzl 1997](https://users.pfw.edu/isiorho/G300Eiswirth-Hoetzl_IAH_Nottingham_1997.pdf)). The Chesapeake review drops each basin's share of pipes that lie inundated at a mean invert depth of 9.2 ft (2.8 m) from its exfiltration ([CBP 2025](https://www.chesapeakebay.net/files/documents/Sanitary-Sewer-Exfiltration_July25_JDel_072325.pdf)).
- **The ground.** The fate of what leaks "essentially depends on the geology and mineralogy of the sewer's surrounding": its clay and organic matter, and its pH ([Eiswirth and Hötzl 1997](https://users.pfw.edu/isiorho/G300Eiswirth-Hoetzl_IAH_Nottingham_1997.pdf)).

---

## 3. How far it reaches

**Measured.**
- Below the Rastatt test sewer, the groundwater was plainly sewage-influenced: its specific conductivity was 1,000 µS/cm, with raised potassium, chloride and CO₂ and no oxygen. Wells farther along the profile were not influenced in the same way. The authors conclude that "most of the waste water compounds leaking to the underground are attenuated and biodegraded within the unsaturated zone", so "the groundwater quality is influenced only within a narrow zone next to the sewer leakages" ([Eiswirth and Hötzl 1997](https://users.pfw.edu/isiorho/G300Eiswirth-Hoetzl_IAH_Nottingham_1997.pdf)).
- In two UK sandstone aquifers under cities, sewage-derived bacteria and viruses were regularly found to 60 m depth. Norovirus and Coxsackievirus detections followed the seasonal discharge of viruses to the sewers ([Powell et al. 2003, *Water Research* 37: 339–352](https://researchspace.auckland.ac.nz/handle/2292/8375?show=full); *search summary*). Fissures carry water far faster than the matrix.

**Estimates.**
- Modelling transport from leaky sewers in homogeneous sandstone gave distances of 0.2 to 14 m for pathogenic viruses, for groundwater velocities of 1 to 100 m a year (Joyce et al. 2006, EGU abstract; *search summary*). A companion assessment found the microbiological risk from leaking sewers very low unless the sewer failed catastrophically (Lerner et al. 2006, EGU abstract; *search summary*).

**Practice (regulation, not measurement).** A drinking-water well is kept 50 ft (15 m) from a sewer line in El Dorado County, California ([county setbacks](https://www.eldoradocounty.ca.gov/County-Government/County-Departments/Environmental-Management/Environmental-Health/Septic-System-Components/Septic-System-Minimum-Setback-Requirements)) and in Minnesota. Minnesota allows 20 ft (6 m) for cast iron or plastic pipe that has passed an air test ([Minnesota Department of Health](https://www.health.mn.gov/communities/environment/water/wells/construction/bldgremdemo.html)). Wisconsin's rule is the same 50 ft ([Wisconsin DNR](https://dnr.Wisconsin.gov/sites/default/files/topic/DrinkingWater/Publications/DG102.pdf)). All three are from a *search summary*.

---

## 4. TCE rules (proposed)

These are design priors anchored to the measurements above, not measurements themselves.

- **A sewer leaks along its worn walls, a share of what passes each metre in proportion to that metre's wear.** A load that enters at a home's drain leaks, at each route cell it passes on its way to the outfall, the share `leak_per_km × loss × metres / 1000` of what is left of it, where `loss` is the cell's wall loss (0 new, 1 collapsed).
  - `leak_per_km` 0.1 is an estimate: a 2 km reach at half wear then leaks about 10 per cent, as the tracer study measured on a reach of unknown condition (§1.1).
  - A village sewer of 300 m at half wear then leaks about 1.5 per cent, within the 1–5 per cent the reviews give (§1.1).
  - A new sewer leaks nothing, a simplification of the small allowable leakage and of rehabilitation's 50–90 per cent cut (§1.3).
- **What leaks soaks into the ground at the leaking cell, by the paths a heap's seepage takes.** It goes toward the wells and rivers within the seepage reach and no higher, with the travel time through the ground and decay on the way. The narrow zone (§3) and the 15 m setbacks sit within the 30 m reach already used for heaps.
- **A pipe below the water table lets nothing out.** Where the aquifer's head stands above the invert, a worn wall takes groundwater in instead (§2; built 2026-11-18). The invert's depth, 2.8 m, is an estimate taken from the Chesapeake review's mean invert depth of 9.2 ft, a professional judgement.
- **Not yet:**
  - storms raising leakage above the clogged rate (§2);
  - fissured ground carrying pathogens farther (§3);
  - infiltration's added flow (§1.2), which would need a sewer's water, not only its loads.
