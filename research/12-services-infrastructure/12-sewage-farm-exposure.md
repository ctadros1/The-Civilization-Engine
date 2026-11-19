# What a sewage farm's produce and its workers carry: crops, handling and the guidance, for The Civilization Engine

## Executive conclusion

**Sewage farming is documented well as a practice and as a modern risk, and poorly as a 19th-century health record.** This short note was written for one gap (plan §8, question 19, item 5). ADR-0044 built the sewage farm (§4): a field watered from a sewer takes its load into the field's node. It deferred "produce and workers' exposure on a farm (no food or handling route)". ADR-0045 has since built a food route, which meets that deferral's "Revisit when". Research 05-07 covers enteric disease between people, and 12-10 and 12-11 cover sewage treatment and leakage. None says what a crop grown on sewage, or the people who work it, carry. The findings:

1. **The historical farms were large, long-lived and grew food.**
   - Berlin spread its sewage on *Rieselfelder* from 1876 to 2010, at their height about 10,000 to 12,500 ha, and grew grain and vegetables on them, some sold back to the city.
   - Paris irrigated at Gennevilliers from 1868 to 1869 (115 ha in 1876, more than 300 ha by 1880), then at Achères and beyond.
   - English towns kept farms such as Croydon's at Beddington.
   - Contemporary claims that the farms were healthy come from their promoters, and no reliable 19th-century disease data was found (*uncertain*).
2. **The modern evidence of risk is clear for untreated water and raw-eaten crops.**
   - Consumers of raw vegetables irrigated with untreated wastewater carry more Ascaris.
   - Cholera, typhoid and shigellosis outbreaks have been tied to untreated wastewater use.
   - Diarrhoea risk varies by crop, about twice for some (onion), and none was found for raw vegetables taken all together.
3. **Workers carry more than consumers.** In Mexico's Mezquital Valley, people in farm households irrigating with untreated wastewater had several times the odds of Ascaris infection: about 5.7 for children and 13 for older people. Children had a third more diarrhoea. Storage in two reservoirs in series cut the Ascaris risk substantially, and one reservoir did not. Hookworm enters through bare skin in the field.
4. **The guidance sets the shape: treat, hold, choose the crop, cook.** The WHO 2006 guidelines take a tolerable burden of 10⁻⁶ DALY a person a year. They set at most 1 helminth egg per litre for unrestricted irrigation, and combine treatment with barriers (crop restriction, withholding, washing, cooking).
5. **For TCE's crops,** which are all grain, eaten baked or boiled, cooking is the main barrier. The consumer's route from the crop is small, and the worker's route by hands and soil is the larger (*derived*).

**Conclusion for TCE:** a sewage farm's risk to people is mostly to those who work it, by hands and soil, and little to those who eat its grain once cooked. A crop eaten raw (none in the core content) would carry more. A handling route is not built. TCE's diseases (cholera and bacillary dysentery) pass by water and within a household. No helminth is authored.

Throughout, **measured** or primary findings are kept apart from **estimates**. Most sources were reached only through search-engine summaries in this session. They are marked (*search summary*), and none is load-bearing.

---

## 1. The historical farms

- **Berlin.**
  - Sewage was spread on *Rieselfelder* round the city from 1876, under Hobrecht's radial system (Berlin, *Umweltatlas*, "Rieselfelder": [berlin.de](https://www.berlin.de/umweltatlas/boden/rieselfelder/); English: [berlin.de](https://berlin.de/umweltatlas/en/soil/sewage-farms)). The first field, Osdorf, was connected in 1878 (*search summary*).
  - The fields reached about 12,500 ha in 1928 by the atlas; the English page and Wikipedia say about 10,000 ha, a discrepancy left as found ([Wikipedia, "Sewage farm"](https://en.wikipedia.org/wiki/Sewage_farm)).
  - The last field, Karolinenhöhe, closed in 2010.
  - Grain and vegetables were grown, some sent back to the city's hospitals at Buch (*search summary*: [The Berliner](https://www.the-berliner.com/politics/the-forest-that-grew-out-of-berliners-shit/)).
  - A history of the fields is in [PMC9242145](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9242145/), which was not read in this session.
  - Its soils still hold heavy metals: the atlas's subject. The metals are not part of this note.
- **Paris.**
  - Gennevilliers was irrigated from 1868 to 1869, independent gardeners taking sewer water. The area grew from 115 ha in 1876 to more than 300 ha in 1880, but the farms took too little of the sewage to clean the Seine (*search summary*: [Popular Science Monthly 18](https://kwx.liehendr.com/content/wikisource_en_all_maxi_2022-09/A/Page%3APopular_Science_Monthly_Volume_18.djvu/827)).
  - Later sites were Achères (1895), and Méry-Pierrelaye and Carrières-Triel (1899) (*search summary*: [Ville de Paris](https://patpro.paris.fr/in/rest/annotationSVC/DownloadAttachment/attach_cmsUpload_fc1a89af-a032-4da5-a84d-82985e7710fb); [Musée des égouts](https://musee-egouts.paris.fr/?p=684)).
  - Opponents cited waterlogging and "fevers" at Gennevilliers, and the project was withdrawn for a time (*search summary*: same Popular Science Monthly pages). A study by Danet, Bastin and Garrigou-Desarènes on its results exists but was not read.
- **England.**
  - Croydon's farm at Beddington drained, at times, poorly purified sewage into the Wandle. This is recorded in Medical Officer of Health reports (*search summary*: [Wellcome, London's Pulse](https://wellcomelibrary.org/moh/report/b1978658x/44)).
  - No 19th-century record of the farm workers' health was found (*uncertain*).

**What the history gives:** that the farms existed for decades, grew food, and were contested on health without a settled answer. It gives no quantity.

## 2. Consumers of the produce (modern evidence)

- **Ascaris.**
  - A review table lists a significant Ascaris risk for adult and child consumers where untreated wastewater irrigates. It also lists cholera, typhoid and shigellosis outbreaks tied to untreated use (*search summary*: [Solidarités training note, citing WHO guidance](https://octopus-training.solidarites.org/?p=2167)).
  - Shuval and colleagues (1984) found more Ascaris among Jerusalem's adults and children who ate raw vegetables irrigated with wastewater (*search summary*: cited via [AgEcon](https://ageconsearch.umn.edu/record/127719/files/3.pdf)).
- **Diarrhoea.**
  - Blumenthal and colleagues' review reports no risk from raw vegetables taken all together, and about twice the risk for some (onion) (*search summary*: [IRC, Blumenthal 2000](https://ircwash.org/sites/default/files/Blumenthal-2000-Guidelines.pdf)).
  - Protozoa have been found on wastewater-irrigated vegetables without direct evidence of disease from them.
- **What carries it:** eggs and bacteria on the surface of crops eaten raw, leafy ones most. Cooking, peeling and washing are barriers (*search summary*).

## 3. Those who work it

- **Mezquital Valley, Mexico** (Cifuentes, Blumenthal and colleagues). Farm households irrigating with untreated wastewater, against rain-fed controls:
  - **Ascaris:** about 5.7 times the odds for children and about 13 for older people.
  - **Diarrhoea:** about a third more in children.
  - **Storage:** keeping the water in two reservoirs in series for 2 to 6 months cut Ascaris risk substantially, and one reservoir did not.
  - Sources: *search summary* via [Oxford NDPH](https://www.ndph.ox.ac.uk/publications/484169) and [LSHTM](https://lshtm-eprints-repo-prod.lshtm-eprints.cdl.cosector.com/682293/1/362817.pdf); the 1993 survey in Spanish: [SSWM](https://old2019.sswm.info/sites/default/files/reference_attachments/CIFUENTES%20et%20al%201993.%20Problemas%20de%20salud%20asociados%20al%20riego%20agr%C3%ADcola%20con%20agua%20residual.pdf).
- **Hookworm** enters through the skin of those working barefoot in wet, faecally loaded soil. One study found about three times the odds among exposed farmers, with an interval that includes no effect (*search summary*; its location was not established).
- **What carries it:** hands and soil. Ascaris and Trichuris are swallowed from soil and hands, and hookworm comes through the feet. This is the handling route ADR-0044 deferred.

## 4. The modern guidance, as a check

- **WHO, *Guidelines for the safe use of wastewater, excreta and greywater*, vol. 2 (2006):**
  - a tolerable additional burden of 10⁻⁶ DALY a person a year;
  - at most 1 helminth egg a litre for unrestricted irrigation (salad crops included), which needs 95 to 99.99 % egg removal by treatment;
  - a combination of treatment with health-protection measures after it: crop restriction, a withholding period before harvest, washing and cooking, and protective clothing for workers.
  - Sources: *search summary* via [Mara, White Rose](https://eprints.whiterose.ac.uk/5383/2/mara_who.pdf) and [FAO](https://www.fao.org/4/T0551E/t0551e04.htm). The guideline's own tables were not read.
- **Ponds:** waste stabilisation ponds remove nearly all helminth eggs within days (*search summary*), as the Mezquital reservoirs did over months.

## 5. What it depends on (qualitative)

- **Treatment before the field:** none, settling (TCE's settling tank, ADR-0041 §4), or storage in series.
- **The crop and how it is eaten:** grain baked or boiled carries little to the eater; leaves eaten raw carry most.
- **The time from the last watering to harvest:** pathogens die on the crop and in the soil over days to weeks (*qualitative*; no figure read here).
- **How the work is done:** bare hands and feet, and whether workers wash before eating.
- **The pathogen:** bacteria and viruses die faster than helminth eggs, which persist in soil for months (*qualitative, widely stated*).

---

## 6. TCE rules (proposed)

- **Kept as built:** a sewage farm's field takes the load of the sewage it is watered with (ADR-0044 §4). What it holds is in the disease-left accounting by its household.
- **Produce:** TCE's crops are all grain, eaten baked or boiled. Its route to the eater is through ADR-0045's food route only if a load passes from field to harvest and survives cooking. By §2 and §4 the cooking barrier makes this small (*design prior*: loads on grain from a sewage farm are cut by cooking as boiling and baking cut them). A crop eaten raw, if authored, would carry more.
- **Workers:** the larger route is handling. Hours worked on a loaded field dose the worker by the field's load (§3; *design prior*, unquantified here). TCE has no such route: its diseases pass by water, within a household and in company. A handling route is a new route of a disease, so it is a model decision (ADR-0021's routes).
- **Barriers the polity may make:** a withholding period, a crop restriction, or settling before the farm, as laws (§4). They are not authored; they are for later slices, after a route exists.
- **Not here:** helminths (no such disease in the core content), metals in farm soils (no harm model; ADR-0038's revisit).
- **Uncertain throughout:** most figures are modern, regional and from summaries. Treat every rule as a design prior.
