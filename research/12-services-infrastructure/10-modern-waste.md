# Modern waste: refuse, collection, disposal, sewage treatment and industrial residues for The Civilization Engine

## Executive conclusion

**Model modern waste as material that a household, works or street produces, that someone carries somewhere for a price, and that keeps existing wherever it ends up. Do not model it as a cleanliness score, a "waste" stat or a pollution radius.** 12-02 set out the chain from production to exposure for excreta and early refuse (12-02 §1.4, §5.2). This report covers what changes between about 1800 and 1960, when cities burned coal, kept horses, laid water-carriage sewers and began making industrial by-products in bulk:

1. **What a household throws away follows its fuel, its food and its income, not its headcount.** Around 1900 most urban refuse by weight was coal ash. A Manhattan resident discarded roughly 1.85 kg a day, of which about 83 % was ash ([FEE, citing Melosi](https://fee.org/articles/are-we-burying-ourselves-in-garbage/)). When coal left the home, refuse got lighter and bulkier. When packaging arrived, the mass rose again.
2. **Collection is the expensive part.** Today it takes about half to two-thirds of solid-waste spending ([Journal of Environmental Health and Sustainable Development](https://jehsd.ssu.ac.ir/article-1-288-en.pdf)). In nineteenth-century records it was likewise paid by the load and by the round. Collection is a logistics job with a vehicle, a crew, a round and a destination.
3. **Every disposal route is a site with capacity, cost and neighbours.** Open dumps, water dumping, controlled tips, sanitary landfills, destructors and sewage farms each move the problem somewhere and change its form. The cheapest route wins until a neighbour, a court or a disease makes it expensive.
4. **Salvage is a trade whose sign can flip.** London's dust contractors once paid for the right to collect. Within a generation, parishes were paying them to take it away ([Mayhew, *London Labour and the London Poor*, vol. 2](https://dl.tufts.edu/teiviewer/parent/rv043431c/chapter/c7s10); [Velis, Wilson and Cheeseman 2009](https://pubmed.ncbi.nlm.nih.gov/19121575/)). Neither the amount nor the kind of refuse had to change for this to happen; only the prices of what it competed with did.
5. **Industrial residues are by-products of recipes TCE already has.** Coal gas leaves coke, tar, ammoniacal liquor and spent purifier lime. Iron smelting leaves slag, and every coal fire leaves ash and clinker. Closing each recipe's mass balance is the cheapest way to make these residues exist.

Throughout, **observed** history is kept apart from **proposed TCE rules**. Dated milestones are examples of what can happen when particular prices, laws and techniques meet, not era gates.

---

## 1. Mechanisms: causal rules TCE can implement

### 1.1 Refuse is a residue of consumption, and its make-up follows fuel, food, packaging and income

**Observed.**
- Melosi's figures for Manhattan, 1900–1920, give an annual average per resident of about **160 lb of garbage (food waste), 1,230 lb of ashes and 97 lb of rubbish**. That totals 1,487 lb, about 674 kg a year or 1.85 kg a day ([FEE, citing Melosi](https://fee.org/articles/are-we-burying-ourselves-in-garbage/); [Melosi, *Garbage in the Cities*](https://upittpress.org/books/9780822958574/)). The Federal Reserve Bank of Boston puts Manhattan's coal ash alone at about 3.3 lb (1.5 kg) a person a day in the early 1900s ([Boston Fed](https://www.bostonfed.org/publications/regional-review/2002/quarter-1/what-a-waste.aspx)).
- In Britain, a Department of the Environment review found that ash and cinder fell from about **91 % to 57 %** of town refuse between about 1919 and 1935. Average household refuse fell from **17 kg a week at 290 kg/m³ in 1935 to 13.2 kg a week at 157 kg/m³ in 1968** ([DoE 1971, cited in "Consumers, Waste and the Throwaway Society Thesis" (2013)](http://article.sapub.org/10.5923.j.ijas.20130302.02.html)). The weight fell while the density nearly halved, so the volume to be carried hardly fell at all.
- Tim Cooper's evidence to Parliament describes dust and cinders as over half of household waste in the early twentieth century. Dust declined after the war, when gas and electricity replaced coal for household heating ([UK Parliament, written evidence](https://publications.parliament.uk/pa/cm200910/cmselect/cmenvfru/230/230we14.htm)). A 1958 trade report compared one old tenement with one new flat: the tenement's refuse was about a third dust and nearly a third cinders, while the flat's had almost no cinders ([Commercial Motor, 1958](https://archive.commercialmotor.com/article/21st-november-1958/57/cost-deters-dustless-refuse-systems)).
- Today, food and green waste is **32 %** of waste in high-income countries, against **53 %** in middle-income and **57 %** in low-income countries ([World Bank, What a Waste](https://datatopics.worldbank.org/what-a-waste/trends_in_solid_waste_management.html)).

**Why fuel matters so much.** Coal averaged about **11.3 % ash** in a USGS national sample ([USGS OF-76-468](https://pubs.usgs.gov/of/1976/0468/report.pdf)). Central European wood holds about **0.3–1.0 %** ([Luke, Finland](https://jukuri.luke.fi/bitstream/10024/520771/1/951-40-0897-9.pdf)). Switching the same household from wood to coal therefore multiplies its fuel residue per kilogram burned by roughly ten to thirty. Domestic grates also leave unburnt cinders, which dust-yards sold as "breeze".

**TCE rule (proposed).** Refuse is produced by events that already exist:
- burning a fuel leaves that fuel's ash share (a property of the fuel good);
- eating and preparing food leaves an inedible share;
- using up a good leaves its wrapping or worn-out remains.

Composition is never a per-capita constant by era. A household that switches from firewood to coal changes its refuse without any rule about industrialisation.

### 1.2 Keep refuse streams apart where they behave differently

12-02 §1.1 already separates feces, urine, wastewater, organic refuse, ash and manure. For modern waste, five streams need distinct behaviour:

| Stream | What makes it different | What it can become |
| --- | --- | --- |
| **Ash, cinder and clinker** | Dense, inert and does not decay; some unburnt fuel | Brick material and kiln fuel (London "soil" and "breeze"), fill, paving ([Victorian London: dustmen](https://www.victorianlondon.org/professions/dustmen.htm); [Hornsey Historical Society](https://hornseyhistorical.org.uk/smithfield-squares-fascinating-past-part-two/)) |
| **Garbage (putrescible food waste)** | Rots within days; feeds flies and rats; smells | Hog feed, grease and fertiliser by "reduction", compost |
| **Rubbish (paper, rags, bones, metal, glass)** | Light, bulky, sortable | Salvage goods whose value follows the market (§1.6) |
| **Street dirt and horse manure** | Produced on public ways; manure breeds flies | Manure sold to farms when demand allows |
| **Trade and industrial residues** | Made at works in tonnes, often toxic or caustic | Some saleable (coke, tar, slag); the rest lies on works land or reaches rivers (§1.8) |

**Observed.** George Waring's New York department (1895–1898) required households to keep garbage, ashes and rubbish in separate receptacles, so that each could go to a different route ([ASCE: George Waring](https://www.asce.org/about-civil-engineering/history-and-heritage/notable-civil-engineers/george-edwin-waring); [Long Island History Journal](https://lihj.cc.stonybrook.edu/2015/articles/olsen/)). Paris's 1883–1884 decrees required building owners to provide covered containers. One account dates a three-container sort to March 1884, but sources disagree on which decree did what ([Eugène Poubelle](https://en.wikipedia.org/wiki/Eug%C3%A8ne_Poubelle)).

**TCE rule (proposed).** Separation is a choice made at the building, at a cost in containers and attention. It is worth making only where the routes downstream differ in price. Mixed refuse is cheaper to collect and harder to sell.

### 1.3 Storage at the building sets the nuisance clock

**Observed.** Under warm summer conditions a house fly can complete its life cycle in **7–10 days** ([University of Florida IFAS](https://edis.ifas.ufl.edu/publication/IN205/pdf)). Depending on temperature it can take anywhere from 6 to 42 days (WHO vector-control manual chapter, [via Scribd](https://www.scribd.com/document/427922655/vector302to323-pdf)). Flies breed in partly composted manure and garbage, and lidded containers and dry manure are standard control measures (same sources). Joel Tarr reports that Rochester's health officials estimated in 1900 that the city's 15,000 horses bred about **sixteen billion flies a year** ([Tarr, *American Heritage*, 1971](https://www.americanheritage.com/urban-pollution-many-long-years-ago)). That is a local estimate, not a constant.

**TCE rule (proposed).**
- Each building keeps its refuse in a store: an ashpit, a bin or a heap, with a capacity and a lid or no lid.
- Putrescible refuse that waits longer than a temperature-dependent breeding time becomes fly habitat. Until TCE has insect vectors, this is recorded and shown, never converted into sickness (12-02 §5.4: "do not use a rat-population meter as a substitute").
- Ash in the same store does not rot. A collection interval that suits ash (monthly) does not suit garbage in summer (under a week). This is one reason cities separated streams or collected more often.

### 1.4 Collection is a paid run with a vehicle, a round and a destination

12-02 §1.4 established that collection moves a load and does not delete it. The modern changes are scale, vehicles and money.

**Observed.**
- **Pay by the load.** In the 1850s London dustmen were paid about eightpence a cartload ([NPR excerpt of Jackson, *Dirty Old London*](https://www.npr.org/2010/01/01/392332238/excerpt-dirty-old-london)). A later account estimates about three loads of dust per house per year ([Victorian London](https://www.victorianlondon.org/professions/dustmen.htm)). A London Medical Officer of Health report gives a contract cost averaging **3s 8d a load**, at about 1.4 loads per house per year ([London's Pulse](https://wellcomelibrary.org/moh/report/b18039856/25)).
- **Vehicles changed capacity and volume.** Pre-compaction bucket-loader trucks usually held **6–10 cubic yards**. Montreal bought 10-cubic-yard Garwood Load-Packers in 1937. The packer's innovation was compaction: more mass per trip ([Classic Refuse Trucks](https://www.classicrefusetrucks.com/albums/GW/GW03.html); [Garwood Load Packer](https://en.wikipedia.org/wiki/Garwood_Load_Packer)). As refuse got lighter and bulkier (§1.1), volume rather than mass became the binding limit.
- **Collection dominates cost.** Collection takes about **50–70 %** of total solid-waste costs in one review ([JEHSD](https://jehsd.ssu.ac.ir/article-1-288-en.pdf)). A World Bank-linked annex reports about 70 % in industrialised countries and up to 95 % in developing countries ([IISc annex](https://wgbis.ces.iisc.ac.in/energy/paper/privatesector/annex/collection.html)). Transport alone runs about **US$20–50 a tonne** ([World Bank](https://datatopics.worldbank.org/what-a-waste/trends_in_solid_waste_management.html)).

**TCE rule (proposed).** A collection run is posted by whoever does the work: a household firm, a contractor or a polity's crew. It visits a list of buildings, loads up to the vehicle's mass or volume limit, and must unload at a named destination before its next load. Its cost is hours, vehicle wear and fuel, all of which TCE already prices. A longer haul to the tip raises the cost of every load. Nothing is collected without a destination willing or obliged to receive it.

### 1.5 Each disposal route is a site with capacity, cost and a neighbourhood

| Route | What it does physically | Cost structure | Who suffers |
| --- | --- | --- | --- |
| **Open dump** | Piles refuse on land; often burned; exposed to rain | Land and haul only | Neighbours: smoke, flies, rats, runoff, leachate to wells (12-02 §1.3) |
| **Water or sea dumping** | Moves refuse to a river or the sea | Barge or cart haul | Downstream and down-coast parties |
| **Controlled tip / sanitary landfill** | Spreads, compacts and covers refuse daily | Land, cover soil, compaction labour | Less exposure; leachate and gas over decades |
| **Destructor / incinerator** | Burns refuse to clinker, ash and heat | Plant, stoking labour, maintenance | Smoke and dust nearby; clinker must still go somewhere |
| **Reduction plant / hog feeding** | Turns garbage into grease, fertiliser or pork | Plant or herd; transport | Smell; disease through feed (§1.6) |

**Observed: dumping in water moves the problem to someone with standing.** In 1931 the US Supreme Court found that New York City's ocean dumping created a nuisance on New Jersey's shores ([*New Jersey v. City of New York*, 283 U.S. 473](https://www.law.cornell.edu/supremecourt/text/283/473)). After New York missed the first deadline, the Court in 1933 set 1 July 1934 as the final date, with a $5,000 daily fine ([290 U.S. 237](https://www.law.cornell.edu/supremecourt/text/290/237)).

**Observed: destructors were a recipe that had to compete on cost.**
- The Nottingham "Destructor" (Fryer, built by Manlove, Alliott & Co.) began systematic refuse burning in 1874 ([Manlove, Alliott & Co.](https://en.wikipedia.org/wiki/Manlove,_Alliott_%26_Co._Ltd.)).
- The 1911 *Britannica* reported burning costs from 24 English towns of **6d to 2s a ton, averaging about 1s 1½d**, excluding interest on the works. It gave **1–2 lb of water evaporated per lb of refuse**, and a fuel value for domestic refuse of **1/10 to 1/5** that of coal ([EB1911, "Destructors"](https://www.gutenberg.org/files/30073/30073-h/30073-h.htm)).
- Seattle's destructors of 1908 were soon abandoned "in favor of the cheaper method of sanitary land fills" ([HistoryLink](https://www.historylink.org/File/3155)). Edinburgh closed its Powderhall destructor in 1919: the plant was worn out and could handle only 17 % of the city's refuse ([Threadinburgh](https://threadinburgh.scot/2022/09/02/the-thread-about-powderhall-its-origins-railway-station-the-corporation-destructor-and-its-sporting-heritage/)).

**Observed: landfill was an engineering practice before it was a regulated one.**
- Controlled tipping was promoted in Britain from the 1920s. Its advocates had to "recuperate" a practice the profession had called insanitary, and it became local authorities' normal recourse after the Second World War ([Cooper 2010](https://journals.sagepub.com/doi/10.1068/a42120)).
- Fresno's municipal landfill (1937), designed by Jean Vincenz after studying British controlled tipping, combined trenches, compaction and daily cover; Vincenz judged compaction the most important. Fresno had declined to renew a private incinerator franchise ([NRHP nomination summary](https://www.historicfresno.org/nrhp/landfill.htm); [Fresno Municipal Sanitary Landfill](https://en.wikipedia.org/wiki/Fresno_Municipal_Sanitary_Landfill)).
- Even so, a US federal survey later judged **94 % of land disposal sites inadequate** (no daily cover, or open burning) and **75 % of incinerators inadequate** ([GAO report, citing the federal solid-waste office](https://webarchive.library.unt.edu/eot2008/20081012095612/http://archive.gao.gov/f0202/094056.pdf)).

**TCE rule (proposed).** A disposal site is a place with:
- remaining volume;
- state: open, covered or burning;
- a drainage and groundwater connection (12-02 §5.3);
- an owner, and a price or obligation to receive.

Covering costs earth and hours, and compaction costs a tool's work. Both stretch capacity and cut exposure. An incinerator is a recipe: refuse in; clinker, ash, heat and smoke out. It runs only where someone weighs its costs against the haul and land it saves.

### 1.6 Salvage is a trade whose worth follows the prices of what it replaces

**Observed: London dust.** London's dust-yards recovered nearly all household "dust" because brickmakers bought the fine ash ("soil") and the cinders ("breeze") ([Velis et al. 2009](https://pubmed.ncbi.nlm.nih.gov/19121575/); [NPR excerpt](https://www.npr.org/2010/01/01/392332238/excerpt-dirty-old-london)). By Mayhew's time the trade had reversed: "All the metropolitan parishes now pay the contractors various amounts for the removal of the dust." The contractors explained that brick-dust prices had fallen because "there are fewer brickfields than formerly near London, as they have been nearly all built over" ([Mayhew vol. 2](https://dl.tufts.edu/teiviewer/parent/rv043431c/chapter/c7s10)). **A growing city raised the haul distance to its own market for ash.**

**Observed: substitutes and law.**
- **Rags against wood pulp.** Wood pulp displaced rag paper over roughly the 1870s–1890s, and US rag mills were uncompetitive by 1890 ([Purdue, Charleston Library Conference](https://docs.lib.purdue.edu/cgi/viewcontent.cgi?article=1124&context=charleston); [*Journal of Victorian Culture*](https://academic.oup.com/jvc/article/20/1/34/4095574)). In Paris, the 1883–1884 container decrees were accused of halving ragpickers' income ([Eugène Poubelle](https://en.wikipedia.org/wiki/Eug%C3%A8ne_Poubelle); [Collège de France](https://www.college-de-france.fr/en/agenda/lecture/literary-ragpickers-baudelaire-and-others/baudelaire-and-the-rag-trade)).
- **Garbage as hog feed.** A 1949 survey found **17 of 88** US communities feeding raw garbage to hogs ([DTIC report](https://apps.dtic.mil/sti/tr/pdf/AD0061124.pdf)). Garbage-fed pigs carried trichinae at **11.2 %** against **0.63 %** for grain-fed pigs (1948–1952). Garbage-cooking laws of 1953–1954, passed against vesicular exanthema, cut that sharply ([Pork Checkoff/Iowa State, Trichinae](https://www.porkgateway.iastate.edu/resource/trichinae)). Vesicular exanthema was eradicated in 1959 by enforcing bans on feeding raw garbage ([The Pig Site](https://www.thepigsite.com/disease-guide/vesicular-exanthema-fmd)).
- **The same coupling today.** Cairo's Zabbaleen say they recycle about 80 % of what they collect, against 25 % for the corporate contractors. After Egypt culled all pigs in April 2009, observers reported rotting food piling up in the streets ([SDI](https://sdinet.org/tag/garbage-collection/); [Zabbaleen](https://en.wikipedia.org/wiki/Zabbaleen)).

**Observed: war and patriotism.** In Britain recycling declined before 1914 under hygiene concerns and incineration. Wartime need partly reversed this between 1914 and 1945, but after 1945 economic pressure alone could not sustain it without patriotic drive ([Cooper 2008](https://research-portal.st-andrews.ac.uk/en/publications/challenging-the-refuse-revolution-war-waste-and-the-rediscovery-o)).

**TCE rule (proposed).** Salvage fractions are ordinary goods (ash, rags, bones, scrap iron, later paper and glass) with market prices. A collector's margin is 12-02 §5.5's formula, unchanged. Whether a household is paid for its refuse, gives it away, or pays to be rid of it emerges from those prices, the haul and the alternatives. A new substitute, a law or a lost market can flip the sign without anyone deciding that "recycling ends".

### 1.7 Water-carriage sewerage relocates the load; treatment works transform part of it

12-02 §1.5 covers sewers as conveyance. The modern additions are treatment and the residues it creates.

**Observed.**
- **Sewers outpaced treatment.** By 1905 more than 95 % of the US urban population discharged its wastewater untreated into waterways ([SSRN, *The Evolution of Water Pollution Control in the United States*](https://biotech.law.lsu.edu/blog/SSRN-id554223.pdf)). Tarr and colleagues document that newly sewered population grew faster than treated population into the 1920s ([Tarr et al., *Water and Wastes*](https://www.researchgate.net/publication/269660959_Water_and_Wastes_A_Retrospective_Assessment_of_Wastewater_Technology_in_the_United_States_1800-1932)). That growth figure comes from a search summary and was not checked against the full text.
- **Sewage farms.** An 1891 account gives Berlin's sewage farms as about **19,000 acres serving about 1.5 million people** ([*The Sewage Farms of Berlin*](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9242145/)). Berlin's farms peaked at about **10,000 ha in 1928** and fell to about 1,250 ha by 1992 once treatment plants replaced them. They left contaminated soils behind ([Berlin Environmental Atlas](https://www.berlin.de/umweltatlas/en/soil/sewage-farms/)). Freiburg bought **500 ha** in 1891 for a city of over 50,000 ([Rieselfeld](https://en.wikipedia.org/wiki/Rieselfeld)).
- **Activated sludge.** Ardern and Lockett at Davyhulme (Manchester) named the "activated sludge" process in a paper of April 1914 ([Grace's Guide: W. T. Lockett](https://www.gracesguide.co.uk/William_T._Lockett)).
- **Standards tied to the receiving water.** The Royal Commission on Sewage Disposal recommended effluent of at most **20 mg/L BOD and 30 mg/L suspended solids where at least eightfold dilution was available** ([Biochemical oxygen demand](https://en.wikipedia.org/wiki/Biochemical_oxygen_demand); [Royal Commission on Sewage Disposal](https://en.wikipedia.org/wiki/Royal_Commission_on_Sewage_Disposal)). Sources disagree on whether the eighth (1912) or ninth report set this.
- **Modern definitions.** EU law defines a population equivalent as **60 g BOD₅ a day**, and primary treatment as settling that removes at least **20 % of BOD₅ and 50 % of suspended solids** ([Directive 91/271/EEC](https://eur-lex.europa.eu/eli/dir/1991/271/oj/eng)).

**TCE rule (proposed).** A treatment works is a set of recipes acting on the contagion and nutrient loads 12-02 §5.2 already tracks:
- settling splits a flow into a lighter effluent and a **sludge** good;
- a sewage farm is a field receiving load (12-02 §1.6) with an area limit;
- biological treatment (filter or aeration) cuts load further, at a cost in plant, power or land.

The sludge is a residue that must itself be carried, dried, spread, dumped or burned. A works that settles without anywhere to send its sludge has moved the problem a second time.

### 1.8 Industrial residues are the other outputs of recipes

**Observed.**
- **Gasworks.** Gas coal could yield up to about **10,000 ft³ (≈ 283 m³) of gas a ton** ([Coal gas](https://www.wikiwand.com/en/Coal_gas)). The by-products were:
  - about **70 t of coke per 100 t** of coal, and roughly **7.5 gallons of tar a ton**, about 60 % of which ended as pitch ([GANSG: Gas Works](https://www.igg.org.uk/gansg/12-linind/gasworks.htm));
  - **10 gallons a ton**, another trade estimate of tar ([Coloriasto](https://coloriasto.blogspot.com/2016/12/the-chemistry-of-dyestuff-tar.html));
  - ammoniacal liquor at about **24–33 gallons a ton** in a UK statistics table seen only in a search excerpt;
  - spent purifier material. Lime, and later iron oxide, became "spent" carrying cyanide compounds. "Foul lime" held about 6 % cyanides, and spent oxide ("blue billy") was often dumped on any land near the works or in watercourses ([Blue billy](https://en.wikipedia.org/wiki/Blue_billy); [Legal Futures](https://www.legalfutures.co.uk/associate-news/blue-billy)).
- **Iron.** With 60–66 % iron ores, blast furnaces make about **0.22–0.37 t of slag per tonne** of pig iron. Lower-grade ores can reach about **1 t per tonne**. The US national average in 1959 was slightly under 1 t of slag per 2 t of pig iron. Early-nineteenth-century slag output outpaced its uses; later it went to road base, railway ballast and cement ([USGS slag yearbook](https://webarchive.library.unt.edu/eot2008/20080916105323/http:/minerals.usgs.gov/minerals/pubs/commodity/iron_&_steel_slag/myb1-2006-fesla.pdf); [Slag](https://en.wikipedia.org/wiki/Slag)).
- **Trade effluent and the law.** Britain's Rivers Pollution Prevention Act 1876 let polluters plead "best practicable and available means". In 1884 *The Spectator* called it "a dead-letter" ([*The Spectator*, 1884](https://archive.spectator.co.uk/article/13th-september-1884/9/our-rivers); [Hansard 1950](https://api.parliament.uk/historic-hansard/commons/1950/nov/27/rivers-prevention-of-pollution-bill)). The Public Health (Drainage of Trade Premises) Act 1937 gave trade effluent a route into public sewers on notice to the local authority ([vLex](https://vlex.co.uk/vid/861282440)).
- **Tanneries and dyeworks.** This review found no quantitative source for tannery or dye-waste loads. Treat them as recipe outputs with authored compositions (§2.4).

**TCE rule (proposed).** Every recipe that transforms mass should account for all of it. A residue is an output good at the works. It is sold where a market exists (coke, tar, slag), kept on the works' land, or carried to a site or river under the same rules as household refuse (§1.5). Legacy contamination is just residue that stayed where it was put.

### 1.9 Nuisances and exposures act through specific channels

Keep each nuisance a separate, traceable consequence, never a neighbourhood score:

| Nuisance | Physical source | Channel TCE can use |
| --- | --- | --- |
| Contaminated wells and rivers | Dump, tip or farm leachate; sewer outfalls | 12-02 §5.3's sparse directed links; existing contagion nodes |
| Flies and rats | Putrescible refuse and manure left longer than a breeding cycle | Recorded habitat; sickness only once vectors exist |
| Smoke and dust | Destructors, burning dumps, coal grates | Seen and smelt by people downwind, as fire is seen |
| Smell | Garbage, reduction plants, sewage farms, gasworks | Seen and smelt by people nearby; can motivate complaints |
| Gas and settlement | Landfill decomposition | Roughly **50 % methane and 50 % CO₂** once anaerobic, typically within a year ([EPA LMOP](https://www.epa.gov/lmop/basic-information-about-landfill-gas)); most within 20 years ([ATSDR](https://webarchive.library.unt.edu/eot2008/20080920130323/http:/www.atsdr.cdc.gov/HAC/landfill/html/ch2.html)) |

Beliefs about what causes harm stay separate from the harm itself (12-02 §1.7). People act on what they see, smell and suspect.

### 1.10 Who pays, who decides, and what triggers public action

**Observed: four kinds of actor.**
- **The occupier.** A law can place the duty on the occupier. The Public Health Act 1875 let a local authority make bye-laws requiring occupiers to remove refuse or cleanse ashpits at set intervals (s. 44) ([Public Health Act 1875](https://www.legislation.gov.uk/ukpga/Vict/38-39/55/section/42/enacted)).
- **The municipality or its contractor.** The same Act said that every local authority "may, and when required by order of the Local Government Board shall", undertake or contract for removing house refuse (s. 42). Under s. 43, an authority that had undertaken the job could be penalised for failing to act on an occupier's written notice (same source). The 1936 Act kept this permissive-then-mandatory form ([Public Health Act 1936](https://www.legislation.gov.uk/ukpga/Geo5and1Edw8/26/49/part/II/crossheading/removal-of-refuse-scavenging-keeping-of-animals-etc/enacted)).
- **The contractor's terms** shift with salvage value: paying for the right to collect, or being paid to do it (§1.6).
- **Courts and higher governments** arbitrate between jurisdictions (*New Jersey v. New York*), or overrule a city (Paris's council was found to have exceeded its jurisdiction, per [a Heidelberg-hosted journal article](https://journals.ub.uni-heidelberg.de/index.php/fr/article/download/52603/46451/164779)).

**Observed triggers for action:**
- a visible nuisance;
- a disease scare tied (correctly or not) to refuse;
- a salvage market collapsing, so that someone must now pay;
- the old tip filling up or being built over;
- a downstream or down-coast party with standing;
- a national survey exposing poor practice (§1.5).

Laws often lag or fail. The 1876 Act was a dead letter. Of the 190 million tons collected in the 1968 US survey, about 77 % still went to some 14,000 open dumps ([North Dakota Law Review, 1973, via CORE](https://core.ac.uk/download/322511127.pdf)).

**TCE rule (proposed).** Refuse laws are ordinary TCE laws: a target behaviour, a responsible actor, a cost, compliance by choice and enforcement only when cases are brought (12-02 §5.5). They answer issues people measure from harms they saw. Passing a law never cleans anything.

---

## 2. Parameters

**Confidence key**, as in 12-02 §2: **H** strong support for the measurement or formal definition; **M** useful but context-dependent, or confirmed only from a secondary or search-excerpt source; **L** thin. A modern measurement with **H** is still **L** when transferred to another era.

### 2.1 Generation and composition

| Parameter | Value | Use and limits | Source; confidence |
| --- | --- | --- | --- |
| Manhattan refuse per resident, 1900–1920 | **160 lb garbage + 1,230 lb ash + 97 lb rubbish a year** (≈ 674 kg, 1.85 kg/day; ash ≈ 83 %) | Coal-heated city; one place; secondhand citation | Melosi via [FEE](https://fee.org/articles/are-we-burying-ourselves-in-garbage/); **M** |
| Manhattan coal ash | **≈ 1.5 kg/person/day** (3.3 lb), early 1900s | Consistent with the row above | [Boston Fed](https://www.bostonfed.org/publications/regional-review/2002/quarter-1/what-a-waste.aspx); **M** |
| UK household refuse | **17 kg/household/week at 290 kg/m³ (1935); 13.2 kg at 157 kg/m³ (1968)** | Per household, not per person | DoE 1971 via [IJAS 2013 article](http://article.sapub.org/10.5923.j.ijas.20130302.02.html); **M** |
| UK ash-and-cinder share | **≈ 91 % (c. 1919) → 57 % (1935)** | Composition shift with fuel and diet | Same; **M** |
| US MSW generation | **2.68 lb (1.22 kg)/person/day, 1960; 4.6 lb (2.09 kg), 1990; 4.9 lb (2.22 kg), 2018** | EPA's MSW definition; not comparable one-for-one with ash-era totals | [EPA ROE](https://webarchive.library.unt.edu/web/20161225062155/https://cfpub.epa.gov/roe/indicator.cfm?i=53); [EPA national overview](https://19january2021snapshot.epa.gov/facts-and-figures-about-materials-waste-and-recycling/national-overview-facts-and-figures-materials_.html); **H** |
| Global MSW today | **0.74 kg/person/day; national range 0.11–4.54** | Modern; spread reflects income and definitions | [World Bank](https://datatopics.worldbank.org/what-a-waste/trends_in_solid_waste_management.html); **H** |
| Food and green share by income | **32 % high; 53 % middle; 57 % low income** | Composition shifts with income | Same; **H** |
| Coal ash content | **11.3 %** US sample mean; resource cutoff 33 % | Varies by seam; grate residue adds unburnt cinder | [USGS](https://pubs.usgs.gov/of/1976/0468/report.pdf); **H** for sample |
| Wood ash content | **0.3–1.0 %** (Central European species) | Tropical woods higher | [Luke](https://jukuri.luke.fi/bitstream/10024/520771/1/951-40-0897-9.pdf); **M** |
| UK domestic coal | **33.5 Mt in 1913** | National total; per-person figure in §2.5 is my arithmetic | [Hansard 1925](https://api.parliament.uk/historic-hansard/written-answers/1925/mar/11/home-consumption); **H** for total |
| Horse manure | **15–30 lb (7–14 kg)/horse/day, about 22 lb (10 kg) average** | Contemporary sanitary experts' estimate | [Tarr 1971](https://www.americanheritage.com/urban-pollution-many-long-years-ago); **M** |

### 2.2 Collection, disposal and treatment

| Parameter | Value | Use and limits | Source; confidence |
| --- | --- | --- | --- |
| Collection share of total cost | **50–70 %** (modern review); ~70 % industrialised, up to 95 % developing | Modern; supports "collection dominates" | [JEHSD](https://jehsd.ssu.ac.ir/article-1-288-en.pdf); [IISc annex](https://wgbis.ces.iisc.ac.in/energy/paper/privatesector/annex/collection.html); **M** |
| Modern system costs | **≥ US$35/t** basic (low income); **US$50–100+/t** advanced; transport **US$20–50/t** | Relative shares only; never import dollars | [What a Waste 2.0](https://openknowledge.worldbank.org/entities/publication/d3f9d45e-115f-559b-b14f-28552410e90a/full); [World Bank](https://datatopics.worldbank.org/what-a-waste/trends_in_solid_waste_management.html); **M** |
| London dust collection | **8d/load** wage (1850s); **≈ 3 loads/house/year** (1870s); contract **3s 8d/load**, ≈ 1.4 loads/house/year (one later district) | Orders of magnitude for loads per building | [NPR](https://www.npr.org/2010/01/01/392332238/excerpt-dirty-old-london); [Victorian London](https://www.victorianlondon.org/professions/dustmen.htm); [London's Pulse](https://wellcomelibrary.org/moh/report/b18039856/25); **M/L** |
| Early truck bodies | **6–10 yd³** (4.6–7.6 m³) bucket loaders; 10 yd³ packers (1937) | Volume-limited loads | [Classic Refuse Trucks](https://www.classicrefusetrucks.com/albums/GW/GW03.html); **M** |
| Destructor burning cost | **6d–2s/ton**, mean ≈ 1s 1½d, 24 English towns, excluding interest | Relative to the collection cost per load | [EB1911](https://www.gutenberg.org/files/30073/30073-h/30073-h.htm); **M** |
| Destructor steam | **1–2 lb water evaporated per lb refuse**; refuse worth **1/10–1/5** of coal as fuel | Fuel value of refuse | Same; **M** |
| Destructor electricity | **15–80 kWh/ton** burnt across twenty combined stations | Observed spread; one engine-based estimate gives ~100 | [Period text, Wrocław Digital Library](https://www.dbc.wroc.pl//Content/21815/PDF/023908.pdf); **M** |
| Destructor size | **40–60 t/day** (York); **60 t/day** (Accrington) | Town-scale plants | [York Stories](https://yorkstories.co.uk/the-destructor-notes-from-the-archives/); [Accrington power station](https://en.wikipedia.org/wiki/Accrington_power_station); **M** |
| Incineration reduction | **≈ 70 % of mass; up to 90 % of volume** | Modern plants; residue still to dispose | [ETASR](https://distantreader.org/stacks/journals/etasr/etasr-4370.pdf); **M** |
| Landfill in-place density | **≈ 600–770 kg/m³** (1,007 lb/yd³ measured, Iowa; 1,300 lb/yd³ assumed, Oregon) | Modern compactors; historic tips lower | [Iowa DNR](https://programs.iowadnr.gov/solidwaste/OpenText/DownloadDocument/112534); [Oregon DEQ](https://cms.oregon.gov/deq/ghgp/Documents/LGE-ReportingWasteinPlaceA1.pdf); **M** |
| Landfill gas | **≈ 50 % CH₄ / 50 % CO₂**; anaerobic within ~1 year; most within 20 years | Delayed hazard | [EPA LMOP](https://www.epa.gov/lmop/basic-information-about-landfill-gas); [ATSDR](https://webarchive.library.unt.edu/eot2008/20080920130323/http:/www.atsdr.cdc.gov/HAC/landfill/html/ch2.html); **H/M** |
| Sewage load | **60 g BOD₅/person/day** (definition) | Conventional unit | [Directive 91/271/EEC](https://eur-lex.europa.eu/eli/dir/1991/271/oj/eng); **H** as definition |
| Primary settling | **≥ 20 % BOD₅, ≥ 50 % suspended solids removed** (definition) | Minimum, not typical | Same; **H** |
| Royal Commission standard | **20 mg/L BOD, 30 mg/L SS with ≥ 8× dilution** | Standard tied to the receiving water | [BOD](https://en.wikipedia.org/wiki/Biochemical_oxygen_demand); **M** on report number |
| Sewage farm land | Berlin 1891 ≈ **19,000 acres for ≈ 1.5 M** (≈ 0.5 ha/100 people); Freiburg 1891 **500 ha for > 50,000** (≤ 1 ha/100) | Ratios are my arithmetic | [Berlin 1891](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC9242145/); [Rieselfeld](https://en.wikipedia.org/wiki/Rieselfeld); **M** |

### 2.3 Industrial residues

| Residue | Value | Source; confidence |
| --- | --- | --- |
| Gas per tonne of gas coal | Up to **≈ 10,000 ft³ (283 m³)** | [Coal gas](https://www.wikiwand.com/en/Coal_gas); **M** |
| Coke per tonne of coal (gasworks) | **≈ 0.7 t** | [GANSG](https://www.igg.org.uk/gansg/12-linind/gasworks.htm); **M** |
| Tar per ton of coal | **7.5–10 gallons** (≈ 34–45 L if imperial) | GANSG; [Coloriasto](https://coloriasto.blogspot.com/2016/12/the-chemistry-of-dyestuff-tar.html); **M/L** |
| Ammoniacal liquor | **≈ 24–33 gallons/ton** | UK statistics table in a search excerpt, not verified; **L** |
| Spent purifier lime or oxide | Cyanide-bearing; foul lime ≈ 6 % cyanides | [Blue billy](https://en.wikipedia.org/wiki/Blue_billy); **M** |
| Blast-furnace slag | **0.22–0.37 t/t** pig iron (60–66 % Fe ore); up to **1 t/t** with lower-grade ore | [USGS](https://webarchive.library.unt.edu/eot2008/20080916105323/http:/minerals.usgs.gov/minerals/pubs/commodity/iron_&_steel_slag/myb1-2006-fesla.pdf); **H** modern, **M** historical |

### 2.4 Design priors (not measurements)

| Prior | Starting value | Why |
| --- | --- | --- |
| Unburnt cinder left by a domestic coal grate | **5–10 % of coal burned**, on top of its ash | Brings the coal arithmetic in §2.5 up to the 1935 ash-and-cinder share; to be tested |
| Inedible share of food bought unprepared | **10–25 %** by mass | Garbage stream; no historical source found |
| Breeding time before putrescible refuse becomes fly habitat | **7 days warm, 30 days cool** | From the 7–10 and 6–42 day ranges (§1.3) |
| Cover soil in a controlled tip | **≈ 15 % of tip volume** | Daily cover practice; no historical ratio found |
| Destructor residue (clinker and ash) | **25–35 % of mass in** | Brackets the modern ≈ 30 % |
| Tannery and dye effluent | Author per recipe | No source found |
| Horses per person in a horse-drawn city | **1 per 20–35 people** | New York c. 1900 had roughly 100,000–170,000 horses for about 3.4 million people ([FEE](https://fee.org/articles/the-great-horse-manure-crisis-of-1894/)); the head count is my assumption |

### 2.5 Worked example: a coal-burning town of 5,000

All arithmetic is mine; inputs are from §2.1–§2.4.

- **Household refuse, low case.** At the UK 1935 rate (17 kg/household/week) and an assumed 4 people a household: 0.61 kg/person/day, so **3.0 t/day**. At 290 kg/m³ that is **10.5 m³/day**, of which about 57 % (1.7 t) is ash and cinder.
- **Cross-check from coal.** 33.5 Mt of domestic coal for roughly 45 million people (an assumed population) is about 2.0 kg/person/day. At 11.3 % ash plus a 5–10 % cinder prior, that leaves 0.33–0.43 kg/person/day, close to the 0.35 kg implied above.
- **High case.** At Manhattan's 1.85 kg/person/day: **9.2 t/day**.
- **Horses.** At 1 horse per 25 people, 200 horses × 10 kg = **2 t/day of manure** on the streets and in stables. That is two-thirds of the low-case household refuse.
- **Collection.** TCE's handcart carries 150 kg (`content/core/vehicle/handcart.toml`), so the low case needs **20 handcart loads a day**. The road engine's 4,000 kg payload (`road_engine.toml`) would take it in one load, if volume allowed: 3 t at 290 kg/m³ is 10 m³, above an early truck's 4.6–7.6 m³.
- **Tip capacity.** 1,100 t a year occupies about 1,400–1,800 m³ compacted (600–770 kg/m³), or about 3,800 m³ loose. A 1-ha tip raised 2 m (20,000 m³) lasts about **11–14 years compacted, or about 5 years loose**. Cover at 15 % (prior) is 210–270 m³ a year. At TCE's 8 h/m³ for digging (ADR-0010 §2), that is **1,700–2,200 hours a year**: one person most of the year.
- **Destructor instead.** 3 t/day × 15–80 kWh/t gives **45–240 kWh/day**. TCE's core dynamo serves about 40 kWh an evening (10 kW for four hours; ADR-0036 amendment), so this is one to six such evenings. It still leaves **0.8–1.0 t of clinker a day** to sell or tip.
- **Sewage.** If sewered: 300 kg BOD₅/day. A Berlin-style farm needs **25–50 ha**. Primary settling removes at least 60 kg BOD₅/day into a sludge that someone must carry away.
- **Gasworks.** At 2 t of coal a day (a prior): about 570 m³ of gas, 1.3–1.4 t of coke, 70–90 L of tar, and perhaps 220–300 L of liquor, plus spent purifier lime.

---

## 3. Historical and regional variation

These are examples of configurations TCE should be able to produce, not stages it should impose.

| Place and date | What happened | What it shows for TCE |
| --- | --- | --- |
| London, c. 1800–1850s | Dust-yards recovered nearly all ash-rich refuse for brickmaking; by the 1850s parishes paid contractors to remove it, as nearby brickfields were built over ([Velis et al.](https://pubmed.ncbi.nlm.nih.gov/19121575/); [Mayhew](https://dl.tufts.edu/teiviewer/parent/rv043431c/chapter/c7s10)) | Salvage value depends on haul distance to buyers; city growth can destroy its own market |
| Britain, 1875 | Public Health Act: authorities "may, and when required ... shall" remove house refuse; occupier bye-laws as the alternative ([legislation.gov.uk](https://www.legislation.gov.uk/ukpga/Vict/38-39/55/section/42/enacted)) | Permission and obligation are different laws; duty can fall on occupiers |
| Nottingham 1874; Shoreditch 1897 | First Destructor; the first scheme designed from the start to combine refuse burning with electricity, in a borough handling about 20,000 tons of refuse a year ([Manlove, Alliott](https://en.wikipedia.org/wiki/Manlove,_Alliott_%26_Co._Ltd.); [Municipal Dreams](https://municipaldreams.wordpress.com/2013/12/03/shoreditch_electricity/)) | Burning is a recipe with a second output (power) that needs a buyer; an 1897 commentary noted the scheme depended on selling the electricity (["Electricity and Dust" (1897), via PubMed Central](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC5254103/)) |
| Paris, 1883–1884 | Owners must provide lidded containers; ragpickers' income cut; council partly overruled ([Poubelle](https://en.wikipedia.org/wiki/Eug%C3%A8ne_Poubelle)) | A container rule redistributes salvage between occupants, collectors and pickers |
| New York, 1895–1898 | Waring's three-way source separation; offal and garbage processed at Barren Island ([Long Island History Journal](https://lihj.cc.stonybrook.edu/2015/articles/olsen/)) | Separation pays only where separate routes exist |
| US, 1885–1910s | Governors Island furnace (1885) and early municipal "crematories" ([American City & County](https://www.americancityandcounty.com/waste-management-disposal/garbage-bin-there-done-that)); Seattle destructors abandoned for cheaper fills ([HistoryLink](https://www.historylink.org/File/3155)) | Imported techniques fail where costs or refuse composition differ |
| Berlin, 1870s–1928 | Sewage farms peak at about 10,000 ha; replaced by treatment plants; soils left contaminated ([Berlin.de](https://www.berlin.de/umweltatlas/en/soil/sewage-farms/)) | Land-based treatment scales with land; residues persist |
| Manchester, 1914 | Activated sludge named at Davyhulme ([Grace's Guide](https://www.gracesguide.co.uk/William_T._Lockett)) | Treatment as a technique found by experiment, adopted later and unevenly |
| New York, 1931–1934 | Ocean dumping ended by Supreme Court decree; burning followed ([283 U.S. 473](https://www.law.cornell.edu/supremecourt/text/283/473)) | A party with standing can force a costlier route |
| Britain, 1920s–1950s; Fresno, 1937 | Controlled tipping, then sanitary landfill with compaction and daily cover ([Cooper 2010](https://journals.sagepub.com/doi/10.1068/a42120); [NRHP](https://www.historicfresno.org/nrhp/landfill.htm)) | Practice spreads by engineers' imitation, not by date |
| Britain, 1914–1950 | Wartime salvage revives recycling, which fades after 1945 ([Cooper 2008](https://research-portal.st-andrews.ac.uk/en/publications/challenging-the-refuse-revolution-war-waste-and-the-rediscovery-o)) | Non-price motives can sustain salvage briefly |
| US, 1950s | Garbage-cooking laws and raw-garbage feeding bans against hog disease ([Pork Checkoff](https://www.porkgateway.iastate.edu/resource/trichinae)) | A health law can remove a disposal route |
| Cairo, to 2009 | Zabbaleen recycling tied to pigs; pig cull followed by visible street refuse ([SDI](https://sdinet.org/tag/garbage-collection/)) | The same coupling outside Europe and North America, in living memory |

**Later practice (notes).** After 1960 the main changes were regulatory:
- US municipal landfills became subject to federal design and operating criteria ([40 CFR Part 258](https://www.ecfr.gov/current/title-40/chapter-I/subchapter-I/part-258)).
- The EU required member states to divert biodegradable municipal waste from landfill ([Directive 1999/31/EC](https://eur-lex.europa.eu/eli/dir/1999/31/oj)).
- The EU also phased out sludge disposal to surface waters ([Directive 91/271/EEC](https://eur-lex.europa.eu/eli/dir/1991/271/oj/eng)).

Modern generation and composition vary with income (§2.1). TCE should treat these as further laws and techniques a society may or may not arrive at.

**Transferable variables:** fuel mix, diet and packaging, density, horses per person, haul distance to land and buyers, wages, prices of substitutes (brick materials, paper pulp, feed), the receiving water, and who has standing to complain. Calendar date is not one of them.

---

## 4. Stylized facts and validation targets

| Pattern | Evidence | TCE test |
| --- | --- | --- |
| **Refuse mass falls when coal leaves the home, then rises with packaging and income** | Manhattan ≈ 1.85 kg/day (c. 1900–1920, mostly ash) against US MSW 1.22 kg/day (1960) and 2.22 kg/day (2018); UK 17 → 13.2 kg/household/week as density halves | Switching household fuel from coal to gas or electricity cuts refuse mass and raises volume per tonne, without a rule naming an era |
| **Salvage payments can change sign** | London parishes went from being paid to paying ([Mayhew](https://dl.tufts.edu/teiviewer/parent/rv043431c/chapter/c7s10)) | A fall in ash or rag prices, or a longer haul to buyers, turns a profitable round into a paid service, with no change in refuse |
| **Collection, not disposal, dominates cost** | 50–70 % of modern costs; 1911 burning cost of about 1s a ton against about 3s 8d a load to collect | In a run's ledger, hours and haul exceed site costs unless land is scarce |
| **The cheapest route wins until someone with standing objects** | Ocean dumping ended by court; destructors abandoned for fills | A downstream polity's complaint or case can force a costlier route; absent one, dumping persists |
| **Sewer coverage outruns treatment** | > 95 % of US urban wastewater untreated in 1905 | Sewers relocate load downstream (12-02 §1.5); treatment comes only where a downstream harm is seen and paid for |
| **Laws do not equal practice** | 1876 Act a dead letter; 94 % of US land disposal sites inadequate decades after sanitary landfill was described | Enacting a refuse law changes behaviour only through choice and enforcement |
| **Health law can shut a salvage route** | Garbage-cooking laws; trichinae 11.2 % against 0.63 % | Once husbandry exists, banning raw-garbage feeding raises collection costs and sends more garbage to tips |
| **Horses are a waste source on the scale of households** | 10 kg/horse/day | A horse-drawn town's street manure is comparable to its household refuse; motor vehicles remove it without any waste policy |
| **Residues outlive their makers** | Berlin sewage-farm soils; gasworks blue billy | A closed works leaves residue goods on its plot, still linked to wells |

**Engineering invariants to test independently:**
- mass is conserved through every recipe, run and site;
- no site accepts more than its remaining volume;
- no load is collected without a destination;
- no sickness arises from a heap without an infected source (12-02 §1.1).

---

## 5. Modeling recommendation for TCE

The following is a **proposal**, built on what the kernel already has. File paths are relative to the repository root.

### 5.1 What exists to build on

- **The household midden** (`kernel/crates/civ-agents/src/person.rs`, `Midden`).
  - It is one heap per household, holding `kg`, the time it was last brought up to date, the day it was last carried from the water, and a `site`.
  - It grows by `kg_per_person_day` (0.5 kg of "ash, food waste, sweepings and dung") and decays with a single `half_life_days` (365), carrying nitrogen (`MiddenParams`, `kernel/crates/civ-agents/src/params.rs`; values in `content/core/people/early_farmers.toml` `[midden]`).
  - It is carried to fields in 25 kg loads by `manure_field` (`content/core/activity/manure_field.toml`; `kernel/crates/civ-land/src/fields.rs`).
- **A law that moves middens.** `clear_middens` (`content/core/policy/yourselves_yonder_with_middens.toml`; `kernel/crates/civ-agents/src/population/middens.rs`) answers `source_suspected`. A household within 30 m of an open well or stream carries its heap to a land site at least that far from every source. The heap lies there for `keep_days` (2) before counting as home again. The law is kept by choice, and breaches are not brought before anyone.
- **Siting rules.** ADR-0030 §6's `midden_from_wells` rule template.
- **Contamination as directed transfers.** `Node::Midden`, `Well`, `Reach` and `Store` in `kernel/crates/civ-agents/src/contagion.rs`, with incident text in `kernel/crates/civ-sim/src/frames/incidents.rs`.
- **Recipes with several outputs.** `make_gas` gives 0.14 kg of coal gas and 0.65 kg of coke per kg of coal (`content/core/recipe/make_gas.toml`), so **0.21 kg per kg is unaccounted for**. `smelt_pig_iron_with_coke` turns 2 kg of ore and 3.5 kg of coke into 1 kg of pig, **with no slag**.
- **Lime.** A `quicklime` good and a `burn_lime` recipe exist.
- **Vehicles, runs and fares.** Handcart, wagon and road engine (`content/core/vehicle/`). Posted runs and fares on track and on roads (`kernel/crates/civ-agents/src/population/transit.rs`; ADR-0034 §4–§5; ADR-0036 §7).
- **Ledger channels** such as `Freight`, `Fee` and `Gas` (`kernel/crates/civ-agents/src/ledger.rs`).
- **Engines that burn a named fuel** for shaft work, a dynamo, and evening dispatch (ADR-0033; ADR-0036 and its amendment).
- **Digging** at 8 h/m³ (ADR-0010 §2).
- **Household fuel** is firewood only (`fuel_kg_per_person_day`, `params.rs`).
- **No sewers** exist in the kernel.

### 5.2 Refuse as mass-conserving goods at a building

- **Split the midden by behaviour, not by era.** Keep the existing heap's decaying organic share with its nitrogen and half-life. Add an **inert ash share that never decays**. Later, salvage goods (rags, bones, scrap iron) can be kept as ordinary goods in the household's store. That is a small schema change: a second `kg` field, or a short composition array.
- **Make fuel leave its ash.** Give fuel goods an `ash_share` (content):
  - coal about 0.11 plus a cinder prior (§2.4);
  - firewood about 0.005;
  - coke, charcoal and town gas their own values.

  Household burning, already charged by `fuel_kg_per_person_day`, then adds ash to the heap. A household that switches to coal gets a heavier heap through the existing fuel path. No per-era constant is needed.
- **Keep 0.5 kg/person/day only as the wood-and-dung baseline.** Its non-ash parts should eventually come from food preparation and animals, once those exist.
- **Do not let ash decay away.** The current single 365-day half-life is fine for organic matter, but it would make a coal town's ash vanish. That breaks the 12-02 §5.2 ledger.

### 5.3 Collection as a paid run with a destination

- **Who.** A household firm, or a polity's crew under a law, posts a refuse round using the transit machinery's shape: a vehicle, a list of buildings, a day or days a week, and a **destination**. Handcart and road-engine payloads already bound loads. Add a **volume limit** (m³) to vehicles once refuse density differs by stream.
- **Pay.** The household pays a fee per collection on a new ledger channel (`Refuse`, beside `Gas` and `Fee`; an ADR decides). The collector sells salvage at market or pays the site's gate charge. Signs follow prices (§1.6), using 12-02 §5.5's margin formula.
- **Households decide by comparison.** Each weighs carrying its own heap (as `manure_field` and `clear_middens` already do, in hours), paying for a round, or doing nothing and living with the heap.
- **No round without a destination.** A run whose destination is full or refuses it does not load.

### 5.4 Dumps and tips as sites with capacity and neighbours

- **A disposal site is a place record** with an owner, volume remaining, an open, covered or burning state, and composition by stream. Like a carried midden today, it is a `Node` for the contagion transfers (add `Node::Site`). Runoff and groundwater links then work exactly as they do for middens, with no radius.
- **Covering and compaction are work.**
  - Covering is digging and carrying earth at the existing 8 h/m³.
  - Compaction raises density from loose (157–290 kg/m³) toward 600–770 kg/m³. It needs a roller or engine tool (a design prior until a tool exists).
  - A covered stream stops being reachable by runoff.
- **Siting reuses ADR-0030 §6.** "A tip keeps a distance from wells" is a third `RuleTemplate`, proposed only when people have seen a harm the kernel models (sickness, or water suspected).
- **Water dumping.** Tipping into a river reach adds load to that `Reach`. A downstream polity can then raise it through relations between polities (ADR-0020) as it would any grievance. No special rule is needed.

### 5.5 Incineration as a recipe and a fuel

- **A destructor is an installation** that burns refuse as a low-grade fuel. It reuses ADR-0033's engines-that-burn-a-named-fuel, with refuse's energy content set at 1/10–1/5 of coal's (§2.2).
- **Inputs and outputs.** Mixed refuse and stoking hours in; **clinker and ash at 25–35 % of mass** (a prior), heat, and smoke out.
- **Power.** Driving a dynamo uses ADR-0036's path unchanged. At 15–80 kWh per tonne, a small town's refuse lights a few evenings (§2.5), so a destructor is weighed as a station is: by the connections that would pay for it.
- **Smoke.** Smoke is seen by neighbours, as fire is. No smoke dose model is needed for v0.

### 5.6 Salvage as trade

- Add salvage fractions as **goods** with uses in existing recipes:
  - ash or "breeze" as a partial fuel or temper input to an alternative `fire_bricks` recipe;
  - scrap iron as an input to forging;
  - rags and paper once paper exists.
- Their prices come from the market, and the round's margin from those prices and the haul. **Hog feeding waits for husbandry** (ADR-0029 §2: no beasts). When beasts come, garbage as feed and the health laws that restrict it (§1.6) are the first test.

### 5.7 Close the residue balance of existing recipes

- **`make_gas`.** Add tar (about 0.04–0.05 kg/kg) and ammoniacal liquor (about 0.11–0.15 kg/kg) as outputs. Add a purification step that consumes `quicklime` and leaves `spent_lime`. Tar can be sold or burned. Liquor and spent lime lie at the works or are carried to a site or reach. The 0.21 kg/kg gap closes.
- **Pig-iron recipes.** Add `slag` as an output, scaled to ore grade. TCE's 2:1 ore ratio implies a poorer ore than modern 60–66 % Fe, so a prior of about 0.5–1.0 kg per kg of pig is reasonable. Slag is then usable as fill or way material if a way recipe accepts it.
- **Kept or dumped.** Residues kept on a works plot remain there after the works closes. That is all "legacy contamination" needs to be.
- **Toxic loads.** Until TCE models non-pathogen harm, residues have mass, volume and place, but no dose. Record and show them, and do not invent a toxicity effect.

### 5.8 Sewerage and treatment, when sewers exist

Sewers are not in the kernel. When they are, follow 12-02 §1.5 for conveyance and add treatment as recipes on the existing contagion and nutrient loads (§1.7):

1. A settling tank splits load into effluent and sludge goods.
2. A sewage farm is a field receiving load, limited by area (0.5–1 ha per 100 people as a check).
3. Biological treatment is a later technique that cuts load further, at a cost in plant and power.

Sludge goes through §5.3–§5.4 like any other refuse. The Royal Commission's dilution-linked standard is a natural **law template**: a discharge limit that depends on the receiving reach's flow.

### 5.9 Institutions and triggers

Refuse laws are additions to `PolicyKind`, each answering an issue people measure from what they saw:

- **`refuse_duty`**: occupiers must clear or carry their refuse at set intervals, the 1875 s. 44 model. It extends `clear_middens` from "away from the water" to "at least every N days".
- **`collect_refuse`**: the polity runs or contracts a round paid from the store or by `pay_work`, the s. 42 model, with its destination named in the law.
- **`site_from_wells`** and **`no_tipping_in_water`**: siting rules over harms the kernel models.

Candidate issues include `heaps_seen` (a household sees heaps above some size near home), the existing `source_suspected` and `sickness`, and `tip_full`. Each law is kept by choice, with breaches brought only as cases (ADR-0025). Nothing is cleaned by enactment.

### 5.10 What not to build

- No cleanliness score, district waste stat or pollution radius.
- No era-keyed generation rate.
- No sickness from flies or rats until vectors exist.
- No toxicity effect for residues until a non-pathogen harm model exists.
- No automatic municipal takeover of collection.
- No god tool that cleans a district.

**Defer** sewers and treatment (they wait on a sewer design), husbandry-based hog feeding, packaging, paper and glass goods, landfill gas, and post-1960 regulation.

### 5.11 Guards

- Nothing changes where nobody burns coal, posts a round or builds a site. Every save section's digest stays equal apart from new empty sections (M10's gate clause).
- Mass in equals mass out for every recipe, run and site. The observer can trace any tonne: "Bo's heap went by Ada's round to the Mill Lane tip on 3 June; 0.4 t washed toward Wren's well after rain."

---

## 6. Sources, datasets, and limits of the evidence

### Core sources

- **Generation and composition:** Melosi's *Garbage in the Cities* ([Pittsburgh UP](https://upittpress.org/books/9780822958574/)), quoted here through secondary citations; the UK DoE 1971 series, via [an IJAS 2013 article](http://article.sapub.org/10.5923.j.ijas.20130302.02.html); EPA Facts and Figures ([EPA](https://19january2021snapshot.epa.gov/facts-and-figures-about-materials-waste-and-recycling/national-overview-facts-and-figures-materials_.html)); World Bank What a Waste 2.0 ([World Bank OKR](https://openknowledge.worldbank.org/entities/publication/d3f9d45e-115f-559b-b14f-28552410e90a/full)).
- **Salvage and institutions:** Velis, Wilson and Cheeseman (2009) on London dust-yards ([PubMed](https://pubmed.ncbi.nlm.nih.gov/19121575/)); Mayhew vol. 2 ([Tufts](https://dl.tufts.edu/teiviewer/parent/rv043431c/chapter/c7s10)); Cooper (2008, 2010) on salvage and controlled tipping; the Public Health Acts 1875 and 1936 ([legislation.gov.uk](https://www.legislation.gov.uk/ukpga/Vict/38-39/55/section/42/enacted)).
- **Disposal technology:** *Encyclopaedia Britannica* 1911, "Destructors" ([Gutenberg](https://www.gutenberg.org/files/30073/30073-h/30073-h.htm)); Fresno's NRHP record ([Historic Fresno](https://www.historicfresno.org/nrhp/landfill.htm)); EPA LMOP.
- **Sewage:** Directive 91/271/EEC; the Royal Commission on Sewage Disposal; Berlin's Environmental Atlas; Tarr et al., *Water and Wastes*.
- **Residues:** USGS slag yearbooks; Historic England's *The Manufactured Gas Industry* ([Historic England](https://historicengland.org.uk/research/results/reports/182-2020)), already used by 12-09.

### Datasets worth using

| Resource | Use for TCE | Restriction |
| --- | --- | --- |
| EPA MSW Facts and Figures (1960–) | Generation and composition validation for a modern economy | US definitions; excludes ash-era refuse |
| World Bank What a Waste 2.0 | Income-linked composition and collection coverage | Modern; mixed national definitions |
| UK DoE household-refuse surveys (1935–1968) | Fuel-transition validation: mass, density, ash share | Reached here only through a secondary article |
| USGS iron and steel slag yearbooks | Slag ratios by ore grade | Modern ores |

### Limits of this review

- **Web access.** The web-fetch tool could not reach most hosts (PubMed, NCBI, OSTI, Imperial's repository), so many figures were confirmed only through search-engine excerpts. Those are marked **M** or **L** and should be checked against full texts before an ADR relies on them, especially:
  - the Manhattan 160/1,230/97 lb split;
  - the gas-liquor yield;
  - the Royal Commission's report number;
  - the US sewer-versus-treatment growth figures.
- **No collection productivity was found** (loads per crew-day, haul distances for horse carts). Collection cost should come from TCE's own hours, payloads and route times, not from imported rates.
- **No tannery or dye-waste quantities** were found.

### What should remain uncertain

- **Per-capita refuse before 1900 outside a few cities.** Use fuel, diet and goods to generate it; do not fill gaps with a constant.
- **How dense and how fast historic tips filled.** Loose versus compacted densities differ by a factor of two to four, and historic tipping practice varied by town.
- **Nuisance-to-action thresholds.** How many heaps, flies or complaints move a gathering to act is a behavioural prior to calibrate in play, not a historical fact.
- **Toxicity of residues.** Cyanide in spent oxide, metals in sewage-farm soils and tar compounds are real hazards with no dose model here.
- **Whether destructors "paid".** Period figures mix costs with and without interest, and electricity yields span a factor of five.

**Bottom line:** if TCE tracks what each household and works burns and throws away, who carries it, where it goes, and who can object, it can produce dust-yards that pay and then do not, destructors lighting a few streets, tips that fill and leak, courts that stop dumping, and towns that switch fuel and find their refuse changed. None of it needs to be scripted.
