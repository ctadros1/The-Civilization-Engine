# Textiles, clothing, leather and woodworking

## A simulation-ready production model for The Civilization Engine

**The most important design choice is to model clothing as accumulated labor, not merely a quantity of fiber.** A garment’s cost depends on yarn length and fineness, fabric construction, finishing, cutting and assembly. A reconstruction of the Lendbreen wool tunic, whose original dates to approximately AD 230–390, estimated just over **402 person-hours** per tunic. That is a useful benchmark for one product—not a universal price for ancient clothing. [Tidsskrift](https://tidsskrift.dk/atr/article/view/167008)

Leather and wooden goods introduce a different constraint: **substantial elapsed time and inventory can coexist with relatively little active work.** Traditional heavy vegetable tanning can occupy hides for a year; timber may require months of drying. Conversely, a skilled cooper can assemble a relatively simple bucket remarkably quickly. Treating all three as “crafting time” would produce the wrong economy. [JFJ Baker](https://www.jfjbaker.co.uk/the-process)

For TCE, the essential distinctions are:

| Distinction | Why it matters |
| --- | --- |
| Active person-hours versus elapsed processing time | A hide soaking in a pit occupies capacity and capital, but not a worker continuously. |
| Yarn mass versus yarn length | Fine yarn requires much more length per kilogram; mass alone conceals much of the labor. |
| Making material versus assembling an object | Sewing a shirt from purchased cloth is not equivalent to making that shirt from raw fiber. |
| Wardrobe stock versus annual acquisition | A person can own several garments while commissioning considerably less than one complete outfit annually. |
| Functional adequacy versus prestige | Warmth, coverage, fit, cleanliness, fine workmanship and fashionable appearance are different demands. |
| Know-how versus productive capability | Knowing a process does not supply suitable fibers, trained workers, tools, water, fuel or working capital. |

Throughout this report, **observations and published reconstructions are separated from proposed simulation parameters**. “High confidence” means confidence within the stated scope, not that a value is transferable to every society.

---

# 1. Mechanisms: rules TCE can implement

## 1.1 Textiles are several different production systems

Do not make every material pass through an identical `fiber → thread → cloth` recipe.

| Material family | Production sequence | Important simulation consequences |
| --- | --- | --- |
| **Flax, hemp, nettle and other bast fibers** | Harvest stems or bark → separate fiber bundles, sometimes through retting → remove woody material → splice or spin → weave | Retting is a biological processing stage; over-processing can damage the fiber. Long fibers can be joined by splicing rather than conventional draft spinning. |
| **Wool and other animal hair** | Collect fleece/hair → sort → clean as required → separate and align fibers → spin → weave, knit or felt | Fiber length, fineness, cleanliness and preparation affect both labor and possible products. Not all hair makes equally suitable yarn or felt. |
| **Cotton** | Harvest → remove seeds → clean/open fibers → prepare → spin → weave or knit | Seed removal and preparation are distinct bottlenecks. Short-staple and long-staple cotton should not have identical processing properties. |
| **Silk** | Obtain cocoons → reel suitable continuous filaments, or prepare broken/wild silk for spinning → combine threads → weave | Reeling is not ordinary staple-fiber spinning. Domestic sericulture and processing collected wild silk are different capabilities. |
| **Beaten barkcloth** | Harvest suitable inner bark → soften and beat into sheets → finish | This route bypasses spinning and weaving entirely. |
| **Felt** | Prepare suitable animal fibers → combine moisture, pressure and agitation → shape and dry | Felt does not require yarn or a loom. Fulling woven wool is related physically, but it starts with an already woven fabric. |

Archaeological research has substantially revised older assumptions about early plant textiles: many were made from **spliced fiber bundles**, and some fabrics formerly identified as flax are now identified as tree bast. Uganda’s Baganda barkcloth provides a particularly clear example of a clothing material that never passes through a loom. [Springer](https://link.springer.com/article/10.1007/s12520-018-0677-8)

**Implementation rule:** make “prepared fiber,” “continuous filament,” “spliced yarn” and “beaten sheet” distinct intermediate products. They may converge on similar clothing functions without sharing all prerequisites.

Also distinguish **access to a material** from its domestic production. A settlement can weave imported cotton yarn without growing cotton, and use silk without raising silkworms.

## 1.2 Yarn length should determine much of spinning labor

A useful unit is **tex**, the mass in grams of one kilometer of yarn:

\[
L\_{\text{yarn}}=\frac{10^6 M\_{\text{yarn}}}{\mathrm{tex}}
\]

where \(L\) is meters and \(M\) is kilograms.

Thus, 0.5 kg of 100-tex yarn contains 5,000 m; the same mass at 25 tex contains 20,000 m. These are unit conversions, not historical productivity assumptions.

For a simple woven fabric:

\[
L \approx 100A\,[E(1+c\_w)+P(1+c\_f)]
\]

where:

* \(A\): cloth area in m²;
* \(E\): warp ends per centimeter;
* \(P\): weft picks per centimeter;
* \(c\_w,c\_f\): additional length from yarn bending around the other threads.

Add separate allowances for loom-end waste, sampling, breakage, supplementary pattern threads and pile.

**Implementation rule:** compute spinning work from required meters at the selected yarn specification. Plying adds another operation. Do not apply one fixed “hours per kilogram” to coarse rope, ordinary yarn and extremely fine thread.

The historical cotton-spinning literature explicitly compares output at specified yarn counts; comparisons that omit fineness can confuse increased output with a changed product. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13082)

## 1.3 Looms change both throughput and the feasible product

A loom maintains warp tension and helps separate groups of threads to form a passage for the weft. More elaborate mechanisms can reduce repeated selection work, increase practical width or support complex repeating patterns. Han-period models from Chengdu document sophisticated pattern looms in the second century BCE. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/earliest-evidence-of-pattern-looms-han-dynasty-tomb-models-from-chengdu-china/50DDD53B5889E6FF4C88FD75F0271C33)

For TCE, give each loom:

`maximum_width`, `supported_structures`, `setup_hours`, `pick_rate`, `crew_requirement`, `tension_control`, and `maintenance_condition`.

Then apply these rules:

**Setup is a batch cost.** Producing a long run of similar cloth amortizes warping and setup. Repeated short commissions with different specifications are expensive.

**Weaving speed depends on the fabric.** Dense cloth requires more picks per meter. Hand-selected motifs, discontinuous wefts, embroidery and pile introduce additional operations.

**Width is not free.** Wider work may require different equipment or additional labor.

**Bad yarn slows the loom.** Unevenness and weak warp yarn generate breaks, repairs and rejected cloth.

**Complex patterns do not require an advanced mechanical pattern loom.** Manual selection remains an alternative, often with higher labor. Advanced equipment should improve a capability, not retroactively make earlier artistic achievements impossible.

## 1.4 Dyeing and finishing are productive operations, not cosmetic toggles

Dyeing can occur before spinning, in yarn, or after weaving. These choices produce different design possibilities and different risks of wasting previously invested labor. Fulling, raising a nap, shearing and pressing can transform the appearance and handle of woven wool. [Museum Wales](https://museum.wales/wool/about/the-process/)

Represent dye recipes with:

`compatible_fibers`, `colorant`, `auxiliary_materials`, `water`, `fuel`, `active_work`, `bath_capacity`, `repeat_treatments`, `fastness`, and `failure_risk`.

Do not require a mordant for every dye. Indigo belongs to a different process family from many mordant dyes. South Asian textile production combined dye chemistry with painted mordants, printing and resist techniques; it was not simply a choice of colored thread. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/indian-textiles-trade-and-production)

For fulling, track **area shrinkage and density change**, not creation of additional material. A finished cloth may become smaller and denser while retaining approximately the same fiber mass, less processing losses.

**Simulation consequence:** color, pattern, softness, surface finish and fineness can each raise value through different combinations of materials, knowledge and work.

## 1.5 Clothing assembly depends on geometry

Cloth may be worn as a wrap, woven to useful dimensions, cut into largely rectangular pieces, or extensively shaped and fitted. Andean garments illustrate highly sophisticated textile production in which much of a garment’s design was created in the weaving rather than through extensive subsequent tailoring. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/andean-textiles)

A garment recipe should therefore specify:

`cloth_area`, `cutting_yield`, `seam_length`, `closures`, `lining`, `reinforcement`, `fit_requirement`, and `decoration_work`.

A simple wrap can have little sewing labor but very expensive fabric. A fitted coat made from inexpensive cloth may have substantial assembly labor.

**Do not use “number of garments” as a uniform production unit.** A loincloth, tunic, lined coat, pair of stockings and ceremonial robe are not comparable objects.

## 1.6 Leather: preservation, transformation and assembly

Use a branched chain:

**Fresh hide → preservation → cleaning/fleshing → selected dressing or tanning process → drying/softening/finishing → cutting and assembly.**

Preserve the distinctions between:

* **Rawhide:** cleaned and dried skin, useful for some bindings, coverings and rigid components.
* **Soft-dressed skins and furs:** recipes that retain flexibility, sometimes retaining hair.
* **Vegetable-tanned leather:** a chemically transformed material with properties dependent on tannins, hide thickness and finishing.
* **Alum-tawed and other mineral-treated skins:** alternative products, not interchangeable substitutes in every use.
* **Industrial chrome-tanned leather:** a later chemical-production branch.

Analytical work on Egyptian leather identifies different tanning and coloring materials; a surviving “leather object” does not, by itself, establish which process produced it. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/arcm.12239)

Model hides by **area, thickness, species and defect distribution**, not simply “one hide.” A goat skin and a cattle hide should not yield the same number of shoes.

Fresh hides also connect leather supply to livestock decisions. A hide is generally associated with an animal’s death; fleece can be a recurring output. Their supply responses should differ.

## 1.7 Woodworking: material selection often precedes craft labor

Use:

**Standing tree → selected log or branch → transport → splitting/hewing/sawing → green or seasoned stock → shaping/joinery → finishing.**

The route depends on the object. Some products can be made green; dimensionally demanding assemblies need controlled material condition. Drying time depends on species, thickness, weather, stacking and the target moisture content. USDA drying models explicitly incorporate these variables. [US Forest Service R&D](https://research.fs.usda.gov/treesearch/9684)

Separate at least three skill families:

**Carpentry/joinery:** fitting structural members, furniture and boxes.

**Cooperage:** fitting staves and heads under hoops, with additional demands for liquid-tight containers.

**Turning:** shaping a rotating workpiece; suitable for bowls, handles and repeated round components.

Do not require metal nails for all good wooden construction. Early Neolithic wells in central Europe demonstrate sophisticated joinery made with stone tools. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0051374)

## 1.8 Waiting creates capital requirements

For a stable production system:

\[
\text{Work in progress} \approx
\text{annual throughput}\times\text{mean residence time in years}
\]

A tannery producing 100 hides annually with a one-year processing residence needs approximately 100 hides somewhere in its production pipeline, before allowing for safety stock or failures.

That is a **derived accounting relationship**. It means TCE needs storage capacity, ownership and finance—not a worker standing at each pit for twelve months.

---

# 2. Quantitative parameters

## 2.1 Published benchmarks

These observations are deliberately heterogeneous. Their boundaries are part of the data.

| Process or product | Quantitative benchmark | Scope and qualification | Confidence |
| --- | --- | --- | --- |
| Lendbreen wool tunic reconstruction | **Just over 402 person-hours/tunic** | Reconstructed collection/sorting, yarn production, weaving, finishing and sewing; not a universal ancient garment. | **M**, reconstruction. [Tidsskrift](https://tidsskrift.dk/atr/article/view/167008) |
| Hand-spindle spinning, Borgund reconstruction | **25–33 m/person-hour** | Two experienced craftspeople; specific yarn and reconstruction conditions; fiber preparation excluded. | **M** for case; low transferability. [EXARC](https://exarc.net/issue-2026-3/rev/borgund-curly-pile-textile-reconstruction) |
| Wheel spinning in the same project | **About 214 m/person-hour** | 50 m in 14 minutes; a later spinning-wheel technique used for comparison, not proof of medieval productivity. | **M** for case. [EXARC](https://exarc.net/issue-2026-3/rev/borgund-curly-pile-textile-reconstruction) |
| Cotton spinning, Britain, 1760s | **8 hanks/operative-day at 16 count** | A hank is 840 yards; therefore approximately **0.227 kg/day**. Do not convert to hourly output without a workday assumption. | **M**, reconstructed historical estimate. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13082) |
| Cotton jenny spinning | **25.6 hanks/operative-day in the 1770s; 74.4 in the 1790s**, both at 16 count | Approximately **3.2× and 9.3×** the preceding hand-spinning benchmark at the same count. Improvements continued after initial invention. | **M**. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13082) |
| Modern T-shirt sewing work study | **6.48 standard minutes/garment** | One study’s sewing-operation standard; excludes producing the cloth and is not total factory-chain labor or elapsed lead time. | **M–L**, single case. [European Scientific Journal](https://last.eujournal.org/index.php/esj/article/view/4452/4287) |
| Traditional shoemaking | **At least one pair per maker-day** | Williamsburg describes assembly **from a bundle of prepared parts**; cutting, closing and leather production must not be silently included. | **M**, specialist reconstruction practice. [Colonial Williamsburg](https://www.colonialwilliamsburg.org/events/leather-breeches-maker-shoemaker/) |
| Simple coopered bucket | **About one active hour** | Expert cooper’s account; excludes forestry and seasoning and should not be generalized to large casks. | **M**, firsthand craft testimony. [Colonial Williamsburg Podcasts](https://podcast.history.org/2012/01/02/meet-the-cooper/) |
| Heavy oak-bark leather tanning | **12 months of tannage** | Baker’s process: approximately three months suspended in liquors, followed by nine months layered; finishing extends the total. | **H** for this producer, not all vegetable leather. [JFJ Baker](https://www.jfjbaker.co.uk/the-process) |
| Red-oak boards, nominal 25.4 mm | **70–200 days air drying, green → 20% moisture** | USDA ranges vary with conditions. | **H** within stated engineering conditions. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/32416.pdf) |
| White-oak boards, nominal 25.4 mm | **80–250 days air drying, green → 20% moisture** | Not a universal “one year per inch” rule. | **H** within scope. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/32416.pdf) |
| Kiln drying the same thickness of oak | Red oak **15–28 days**, white oak **20–30 days**, green → 6% moisture | The endpoint differs from the air-drying rows; these are not directly equivalent treatments. | **H** within scope. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/32416.pdf) |

The Borgund project also illustrates why luxury work needs a separate complexity term: extrapolating a small curly-pile sample to a 2 m² textile produced an estimate approaching 2,000 hours before some preparatory operations. That is an **extrapolation**, not a timed manufacture of an entire medieval cloak. [EXARC](https://exarc.net/issue-2026-3/rev/borgund-curly-pile-textile-reconstruction)

## 2.2 Starter parameters for TCE

**Everything in the following table is a proposed calibration prior, not a measured historical average.** The ranges are intentionally broad. Use them for initial implementation and sensitivity testing, then replace them with recipe-specific evidence.

All active-work estimates assume a competent worker, ordinary tools and usable inputs. They exclude upstream production unless explicitly stated.

| Parameter | Suggested initial range | Unit / boundary | Confidence |
| --- | --- | --- | --- |
| Hand-spindle rate, ordinary craft scenarios | **25–150** | m/person-hour; separate calibration by fiber and yarn specification | Low; reconstruction-informed prior |
| Single-wheel spinning rate | **150–800** | m/person-hour; do not apply uniformly across fibers | Low; reconstruction/historical-envelope prior |
| Plain hand weaving | **0.1–0.5** | m²/person-hour, excluding setup | Low |
| Loom setup | **4–20** | person-hours/setup for relatively ordinary work | Low |
| Ordinary adult upper garment | **2–3** | m² of cloth issued to the cutting operation | Low; geometry prior |
| Long garment or substantial wrap | **3–6** | m²/garment | Low |
| Hand cutting and sewing, simple garment | **6–20** | person-hours, from finished cloth | Low |
| More fitted, lined or elaborate garment | **20–60+** | person-hours of assembly; embroidery additional | Low |
| Cutting yield, rectangular construction | **0.85–0.98** | fraction of issued cloth incorporated into garment | Low |
| Cutting yield, shaped construction | **0.65–0.90** | fraction incorporated; scraps remain usable inventory | Low |
| Leather preparation/tanning active labor | **2–8** | person-hours per m² of raw hide; batch residence separate | Very low |
| Hand-assembled ordinary footwear | **6–20** | person-hours/pair from prepared leather and components | Low |
| Small turned bowl | **0.5–3** | person-hours from suitable blank | Very low |
| Simple stool | **2–8** | person-hours from prepared stock | Very low |
| Joined chest | **12–40** | person-hours from prepared stock; size/decoration additional | Very low |
| Liquid-tight cask | **8–24** | person-hours from suitable prepared staves and heads | Very low |

These are **not confidence intervals**. In particular, the leatherworking and furniture values should be treated as placeholders for timed craft trials. Their apparent numerical precision must not become false historical certainty.

For dyes, fulling and tanning, author **batch recipes** rather than universal per-item rates. A partly filled vat should often cost more per unit than a full one.

## 2.3 Worked garment calculation

Consider an **illustrative TCE tunic**, not a reconstruction of a particular archaeological find:

* Cloth issued: **2.5 m²**
* Warp and weft density: **10 threads/cm each**
* Yarn bending allowance: **10%**
* Yarn: **100 tex**

Then:

\[
L=100(2.5)(10+10)(1.1)=5{,}500\text{ m}
\]\[
M=\frac{5{,}500(100)}{10^6}=0.55\text{ kg}
\]

The cloth weighs approximately 220 g/m² before cutting losses.

Assume the following authored labor budget:

| Operation | Person-hours |
| --- | --- |
| Fiber preparation | 20 |
| Loom setup allocated to this garment | 8 |
| Weaving | 15 |
| Finishing | 6 |
| Cutting and sewing | 10 |
| **Subtotal excluding spinning** | **59** |

At 30 m/hour, spindle work is about 183 hours: **242 hours total**.

At 200 m/hour, wheel work is 27.5 hours: **86.5 hours total**.

The spinning operation improved **6.7-fold**, but the completed garment improved only **2.8-fold**. That is the production-chain behavior TCE should reproduce.

This example excludes growing fiber, maintaining animals, transporting inputs and making tools. It also does not establish that wool, linen and cotton have identical preparation or spinning rates.

---

# 3. Clothing consumption: stock, replacement and class

## 3.1 What the evidence actually measures

There is no reliable universal series of “garments consumed per person per year” spanning ancient and preindustrial societies. Surviving evidence includes prescriptions, institutional allowances, household accounts, probate inventories and surviving objects. Those measure different things and represent different populations.

| Evidence | Quantity | Correct interpretation |
| --- | --- | --- |
| **Cato, Roman agricultural prescription, second century BCE** | A tunic and a heavy cloak/blanket every **two years**; old material retained for patching | An enslaver’s provisioning prescription: roughly 0.5 of each item annually. Not an ordinary Roman wardrobe survey or an adequacy standard. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Cato/De_Agricultura/B%2A.html) |
| **Frederick Douglass, recollection of Maryland plantation life, published 1845** | Annual adult allowance: **two coarse linen shirts, summer trousers, winter trousers, a jacket, stockings and shoes** | A specific coercive provisioning regime. Douglass also describes inadequate children’s clothing. This is evidence of deprivation, not a recommended subsistence basket. [Project Gutenberg](https://www.gutenberg.org/files/23/23-h/23-h.htm) |
| **United States household expenditure surveys** | Apparel share approximately **14.0% in 1901**, **4.2% in 2002–03** | Spending shares, not garment counts or physical output. Population and survey definitions also changed. [Bureau of Labor Statistics](https://www.bls.gov/opub/100-years-of-u-s-consumer-spending.pdf) |
| **WRAP UK adult wardrobe research, fieldwork 2021** | Reported average **118 items**, including underwear and hosiery; approximately **26% unworn in the previous year** | A modern national wardrobe-stock benchmark. It is not annual acquisition and not a world average. [Wrap](https://www.wrap.ngo/media-centre/press-releases/nations-wardrobes-hold-16-billion-items-unworn-clothes-people-open-new) |

These examples support a crucial distinction: **new production, ownership and actual use can move independently.**

## 3.2 Recommended demand model

Give garments both a physical state and a social state:

\[
\text{purchase pressure}
=
\text{functional shortfall}
+\text{replacement pressure}
+\text{growth/fit pressure}
+\text{status pressure}
\]

This is a proposed behavioral structure, not an empirically estimated equation.

Functional demand should depend on climate, occupation, mobility, washing/drying arrangements and cultural dress practices. Status demand should depend on institutions and social comparisons.

Track:

**Wear while used.** Work, walking and exposure contribute different damage.

**Storage deterioration.** An unworn item should not suffer the same abrasion as a daily work garment, but neither should storage make it immortal.

**Repair and alteration.** Patching, replacing soles, resewing seams and resizing can preserve usefulness without restoring an object to new condition.

**Transfer.** Inheritance, gifts, wages in kind and second-hand sales move garments between people without creating new cloth.

**Conversion.** Adult clothes can become smaller garments or household cloth; finally, fragments become patches, stuffing or other low-grade material.

For an initial handmade-clothing economy, the following are reasonable **test settings**, not historical findings:

| Household situation | Main garments owned per adult | Newly made main garments acquired annually |
| --- | --- | --- |
| Low-income, predominantly handmade supply | **3–6** | **0.5–2** |
| Middling, predominantly handmade supply | **6–15** | **1–4** |
| Wealthy or institutionally privileged | **15–50+** | **4–12+** |
| Lower-income household with access to industrial cloth/clothing | **6–20** | **2–6** |

Exclude underwear, footwear, blankets and occupational equipment from these counts and model them separately. Change the basket for different clothing systems rather than applying a universal “European outfit.”

These settings should be overridden by simulation outcomes. An impoverished household may retain fewer adequate garments than its desired stock. A wealthy household may spend on extraordinarily labor-intensive fabric without acquiring many additional items.

## 3.3 Textile demand extends far beyond clothing

Create separate demand accounts for bedding, sacks, sails, tents, nets, cordage, furnishing textiles and institutional uniforms.

Likewise, “household goods” should be functional categories. A sleeping surface might be a mat, bedding on a platform or a wooden bedstead. A storage container might be a basket, ceramic vessel, skin bag or cask.

**Do not make every prosperous household converge on the same furniture and clothing inventory.**

---

# 4. Variation across eras and regions

## 4.1 Eras are production configurations, not universal stages

| Context | Historically important pattern | TCE implication |
| --- | --- | --- |
| **Foraging societies** | Wood shaping, cordage and skin working substantially predate agriculture. Evidence includes African woodworking and probable fur/leather-working tools, and Pleistocene European cordage. [Nature](https://www.nature.com/articles/s41586-023-06557-9) | Start with meaningful craft competence. Do not make farming a prerequisite for clothing, containers or wooden structures. |
| **Early farming** | Plant textiles were not necessarily flax, and fiber preparation was not necessarily modern-style spinning. Neolithic communities could also execute sophisticated joinery. [Enlighten Publications](https://eprints.gla.ac.uk/293971/) | Household production can be technically demanding despite simple-looking tools. |
| **Preindustrial societies** | Highly elaborate fabrics, specialized finishing, trade and institutional production existed without industrial machinery. Andean and South Asian traditions demonstrate different combinations. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/andean-textiles) | Quality and artistic complexity must not be locked behind industrial technology. |
| **Industrializing societies** | Different stages mechanized at different times; spinning machinery continued improving after its first invention. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13082) | Bottlenecks migrate. Cheap yarn can coexist with expensive weaving or garment assembly. |
| **Modern systems** | Very short standardized sewing operations coexist with large wardrobes and substantial non-use; synthetic fibers add a chemical-production route. [European Scientific Journal](https://last.eujournal.org/index.php/esj/article/view/4452/4287) | Model distributed supply chains, scale, inventory, fashion turnover and utilization separately. |

## 4.2 Regional contrasts that should survive in the simulation

**South Asia: cotton and finishing expertise.** Textile competitiveness can come from dyeing, printing, material preparation and specialized labor networks—not only from the fastest loom. Indian textile production and trade are particularly useful models for such a system. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/indian-textiles-trade-and-production)

**China: silk and sophisticated loom mechanisms.** Silk production needs an appropriate biological supply chain, while complex pattern-weaving equipment is a separate mechanical capability. Neither should be reduced to a generic “luxury cloth” research bonus. [William Paterson University](https://www.wpunj.edu/articles/news/2022-10-17/online-exhibition-silk-and-the-silk-road-from-hangzhou-to-paterson)

**The Andes: cotton, camelid fiber and textile-centered prestige.** Coastal and highland materials can support complementary trade, while elaborate weaving can embody rank and institutional obligations. Garment construction need not resemble European tailoring. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/andean-textiles)

**East Africa: barkcloth as a viable alternative.** Baganda barkcloth is made by processing and beating inner bark and has served clothing, bedding and other purposes. A society pursuing this route should not be treated as having failed to discover spinning. [UNESCO ICH](https://ich.unesco.org/en/RL/barkcloth-making-in-uganda-00139)

**Arctic environments: sophisticated skin clothing.** A documented Greenlandic caribou-skin jacket sewn with sinew illustrates a material and construction system adapted to a different environment from linen or cotton dress. Such systems require their own skills and material properties. [British Museum](https://www.britishmuseum.org/collection/object/E_Am-1212)

**Inner Asian pastoral settings: felt alongside woven cloth and leather.** Pazyryk finds include felt clothing and elaborate combinations of materials. Mobility does not imply crude clothing or an absence of demanding craft specialization. [Hermitage](https://hermitage.academia.edu/StepanovaElena)

**Europe and the North Atlantic: woolen production and water-powered finishing.** These offer well-documented examples of household work, specialist workshops and powered processing operating in combination. A mill need not replace every domestic stage. [Museum Wales](https://museum.wales/collections/online/object/fb756779-fb3a-3382-90bb-3166ffaf7a03/Esgair-Moel-Woollen-Factory/?field0=string&field1=with_images&index=2299&page=192&value0=museum&value1=1)

These are **ecological and institutional configurations**, not immutable cultural bonuses.

---

# 5. Technology graph

## 5.1 Graph conventions

Use technological knowledge as a set of capabilities. Resources, skilled workers and buildings remain separate requirements for actual production.

In the tables below, **“+” means AND; “OR” means an alternative route**. Prerequisites are proposed TCE design dependencies. Dates are historical annotations, never era gates.

**A date marked “by” is a conservative attestation, not a claim of invention.** Several processes have no securely established global first appearance. It is better to retain that uncertainty than manufacture a precise origin.

## 5.2 Fiber, textile and clothing nodes

| Node | Proposed prerequisites | Approximate earliest evidence / chronology | Concrete unlocks |
| --- | --- | --- | --- |
| **TX01 — Cordage and fiber joining** | Suitable fibers + manual preparation | Three-ply bark-fiber cord at Abri du Maras, France, approximately **52,000–41,000 years ago**. | Binding cord, nets, lashings, sewn joins using appropriate thread; infrastructure for later crafts. [Nature](https://www.nature.com/articles/s41598-020-61839-w) |
| **TX02 — Bast extraction and splicing** | TX01 + suitable stems/bark; water-dependent methods optional | Neolithic Southwest Asia; tree-bast fabrics at Çatalhöyük approximately **6700–6500 BCE**. | Prepared bast, spliced yarn, linen-like cloth routes; retting areas and preparation tools as recipe variants. [Enlighten Publications](https://eprints.gla.ac.uk/293971/) |
| **TX03 — Cotton preparation** | Cotton supply + seed removal/opening knowledge | Neolithic South Asia; ancient Mehrgarh dates need chronological caution. Cotton textiles also occur in early Andean contexts. | Clean cotton fiber; manual ginning, later improved gins; cotton-yarn recipes. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440301907794) |
| **TX04 — Animal-fiber preparation** | Suitable fleece/hair + sorting and separation | Wool textiles are attested in the Near East by the **fourth millennium BCE**; this is not a date for all animal-hair use. | Graded wool/hair, combed or otherwise prepared fiber, differentiated coarse/fine yarn inputs. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/untwisting-beycesultan-hoyuk-the-earliest-evidence-for-nalbinding-and-indigodyed-textiles-in-anatolia/4A00AC3051C50B89F5907F25F03A4B5C) |
| **TX05 — Spindle spinning** | Prepared staple fiber + spindle-making capability | Established in Neolithic traditions; a single secure global invention date is unresolved. | Portable yarn production; spindle and whorl recipes; plied thread as an additional operation. [Future Museum](https://www.futuremuseum.co.uk/collections/arts-crafts/crafts/knitting--crocheting-weaving/spindle-whorl) |
| **TX06 — Basic weaving and loom tensioning** | Continuous yarn/filament from TX01, TX02, TX05 or silk processing + simple frame/tensioning capability | Woven textiles in Neolithic Anatolia, **seventh millennium BCE**; surviving fabric does not identify every loom type. | Plain cloth, narrow bands, ground-, body-tensioned and upright-loom variants, household loom workplaces. [Enlighten Publications](https://eprints.gla.ac.uk/293971/) |
| **TX07 — Silk processing** | Cocoon supply + appropriate preparation/reeling OR staple-spinning method | Chinese silk traditions securely prehistoric; Qianshanyang textiles are third-millennium BCE evidence. Wild silk occurs in Indus contexts approximately **2450–2000 BCE**. | Reeled filaments, spun silk, silk yarn and fabrics; keep domestic sericulture as a separate biological-production capability. [E Zhejiang](https://www.ezhejiang.gov.cn/2026-07/08/c_1196140.htm) |
| **TX08 — Advanced shedding and pattern mechanisms** | TX06 + precise wooden mechanisms + pattern-setting skill | Secure pattern-loom models, Chengdu, China, **second century BCE**; simpler multi-shed techniques are older. | Faster repeated pattern selection, complex repeat structures, specialized loom workshops. Manual pattern work remains possible without this node. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/earliest-evidence-of-pattern-looms-han-dynasty-tomb-models-from-chengdu-china/50DDD53B5889E6FF4C88FD75F0271C33) |
| **TX09 — Beaten barkcloth** | Suitable bark + harvesting/softening + wooden beaters | Origins not securely dated here; documented African traditions include Buganda. | Barkcloth sheets, wraps, bedding and screens; bypasses yarn and looms. [UNESCO ICH](https://ich.unesco.org/en/RL/barkcloth-making-in-uganda-00139) |
| **TX10 — Felting** | Suitable animal fibers + moisture/agitation/pressure | Secure ancient Eurasian examples include Pazyryk, especially **fourth–third centuries BCE**; origins are older or unresolved. | Felt sheets, hats, footwear components, tent and bedding materials. [Hermitage](https://hermitage.academia.edu/StepanovaElena) |
| **TX11 — Natural dye processes** | Compatible textile/fiber + colorant + process-specific auxiliaries and vessels | Indigo-dyed cotton at Huaca Prieta, Peru, approximately **4000 BCE**; other coloring practices differ in age. | Vat-dye and mordant-dye recipe branches, colored yarn/cloth, dye vats and drying space. [Unvis](https://unvis.it/pubmed.ncbi.nlm.nih.gov/27652337) |
| **TX12 — Resist, painted and printed patterning** | TX11 + resist/mordant control + pattern tools | Long preindustrial South Asian traditions; precise earliest dates vary by technique and are not securely resolved here. | Repeated printed patterns, selective dyeing, wax/mud resist, specialized finishing shops. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/indian-textiles-trade-and-production) |
| **TX13 — Fulling and surface finishing** | Appropriate woven wool + wet mechanical treatment; optional raising/shearing tools | Ancient/preindustrial Eurasian practice; global first appearance uncertain. | Dense wool cloth, raised or smooth surfaces, fuller’s workplace, altered fabric dimensions and properties. [Museum Wales](https://museum.wales/wool/about/the-process/) |
| **TX14 — Cutting and sewn assembly** | Suitable sheet material + cutting/piercing tools + thread/sinew | Prehistoric origins; a securely dated sewn leather shoe from Armenia is approximately **3627–3377 BCE**. | Fitted garments, bags, patches, linings and closures. Wrapped clothing does not require this node. [PubMed](https://pubmed.ncbi.nlm.nih.gov/20543959/) |
| **TX15 — Knitting** | Suitable yarn + knitting needles + loop-forming knowledge | Secure surviving knitted examples from Egypt approximately **1100–1300 CE**; do not confuse earlier single-needle looping with knitting. | Elastic hosiery, caps and other shaped knitwear; portable production. [Victoria and Albert Museum](https://www.vam.ac.uk/articles/knitted-underwear) |
| **TX16 — Spindle wheel** | TX05 + wheel/belt mechanisms + suitable carpentry | Asian origins disputed; securely medieval, reaching Europe by roughly the **thirteenth century**. | Faster single-thread spinning where suitable; wheel manufacture and repair. [Smithsonian Institution](https://www.si.edu/object/flax-spinning-wheel%3Anmah_1200991) |
| **TX17 — Flyer-and-bobbin spinning** | TX16 + differential winding mechanism | European development conventionally placed in the **fifteenth century**, with subsequent improvements. | More continuous spinning/winding; distinct equipment and fiber compatibility from a simple spindle wheel. [Cambridge University Press](https://www.cambridge.org/core/journals/early-china/article/abs/spindlewheel-a-chou-chinese-invention/D6D953FAE263938D621CC5E80C1A6FD8) |
| **TX18 — Powered fulling** | TX13 + water power + cams/hammers + mill construction | Medieval Europe; excavated Barrowburn example built **1226–1244** provides a secure benchmark, not the global first. | Fulling mill, processing capacity, miller’s fees, water-right and maintenance demands. [CBA North](https://cbanorth.wordpress.com/2013/12/15/the-coquetdale-community-archaeology-group/) |

**Chronological caution about cotton:** the familiar sixth-millennium BCE date for Mehrgarh cotton comes from an older chronological framework. A 2025 radiocarbon study revised the chronology of the early cemetery. It did not directly redate every cotton-bearing object. Keep the object’s identification, archaeological context and absolute date as separate evidence fields. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440301907794)

## 5.3 Leather and woodworking nodes

| Node | Proposed prerequisites | Approximate earliest evidence / chronology | Concrete unlocks |
| --- | --- | --- | --- |
| **LE01 — Skin preparation and preservation** | Animal skins + scraping/cutting tools | Probable leather/fur-working tools at Contrebandiers Cave, Morocco, **120,000–90,000 years ago**; this is proxy evidence, not identification of a specific tanning chemistry. | Preserved hides, rawhide, pelts; fleshing and drying workplaces. [DOI](https://doi.org/10.1016%2Fj.isci.2021.102988) |
| **LE02 — Soft skin dressing** | LE01 + process-specific fats, smoke or mineral treatments + softening work | Prehistoric/ancient origins; precise dates differ and are poorly resolved. | Flexible skins and furs; alternative dressing recipes rather than one universal “leather” output. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/arcm.12239) |
| **LE03 — Vegetable tanning** | LE01 + tannin-bearing materials + water + containers/pits | Ancient Egyptian objects provide analytical evidence; a single secure global first date is unresolved. | Durable leather grades, tanning pits, bark preparation, currying and finishing operations. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/arcm.12239) |
| **LE04 — Leather cutting and assembly** | Suitable output from LE01/02/03 + piercing/sewing + product pattern knowledge | Sewn footwear securely attested in Armenia around **3500 BCE**; other leather goods have different histories. | Footwear, belts, pouches, straps; harness and bellows when their additional components are available. [PubMed](https://pubmed.ncbi.nlm.nih.gov/20543959/) |
| **WO01 — Splitting, hewing and carving** | Timber + suitable cutting/percussion tools | Structural woodworking at Kalambo Falls, Zambia, at least **476,000 years ago**. | Handles, bowls, posts, frames and shaped wooden components. [Nature](https://www.nature.com/articles/s41586-023-06557-9) |
| **WO02 — Fitted joinery** | WO01 + layout/measurement + fitting skill | Central European wells dated **5469–5098 BCE**, using stone-tool woodworking. | Pegged and fitted structures, furniture frames, boxes and machinery frames; no universal metal-nail prerequisite. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0051374) |
| **WO03 — Board production** | WO01 + riving/hewing OR suitable saw technology | Sophisticated split-timber production is Neolithic; saw-based routes are separate improvements, not prerequisites for every plank. | Boards, panel goods, improved material choices for furniture and containers; waste byproducts. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0051374) |
| **WO04 — Lathe turning** | WO01 + rotating work support + cutting tools | A scholarly reassessment proposes Egyptian lathe components from the **late eighth–early seventh centuries BCE**; interpretation remains qualified. | Turned bowls, handles, pegs and repeated round parts; turner’s workshop. [Ancient Egyptian Interconnections](https://egyptianexpedition.org/articles/two-pivots-of-the-7th-century-bce/) |
| **WO05 — Cooperage** | WO01/03 + accurate stave fitting + hoops; bending/finishing as required | Secure Roman-period European casks; exact earlier beginnings remain uncertain. | Buckets, tubs and casks, with separate dry-goods and liquid-tight recipes. [London Museum](https://www.londonmuseum.org.uk/collections/v/object-26393/barrel/) |

**Seasoning should initially be a material-state process, not a discovery gate.** Wood begins drying without anyone researching it. Better knowledge improves stacking, scheduling, moisture targets and the avoidance of defects.

## 5.4 Industrial and modern extensions

| Node | Prerequisites | Historical benchmark | Unlock |
| --- | --- | --- | --- |
| **IN01 — Multi-spindle jenny** | Spinning + precision frames + coordinated drafting | Britain, **1760s**; patent drawings **1770**. | Multiple hand-powered spindles; larger domestic or workshop output. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/objects/co44855/reproduction-of-hargreavess-original-spinning-jenny-spinning-jennies-spinning-wheels) |
| **IN02 — Roller spinning / water frame** | Prepared cotton + differential rollers + reliable rotary drive | Britain, **1769 onward**. | Continuous powered spinning; factory equipment and power demand. [Industrie Museum](https://www.industriemuseum.be/nl/collectie-item/de-mule-jenny-doorgelicht) |
| **IN03 — Mule spinning** | Roller drafting + intermittent spindle/carriage mechanisms | Britain, **1779**. | Additional yarn-quality/output combinations; later enlargement and partial automation. [Industrie Museum](https://www.industriemuseum.be/nl/collectie-item/de-mule-jenny-doorgelicht) |
| **IN04 — Power weaving** | Loom mechanisms + dependable yarn supply + power transmission | Britain, late **eighteenth–early nineteenth centuries**; adoption was gradual. | Powered weaving stations, machine tending and repair occupations. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13082) |
| **IN05 — Sewing machinery** | Precision metalwork + stitch mechanism + appropriate needles | Practical commercial developments in France from **1830**, followed by further nineteenth-century systems. | Mechanized seams; still requires cutting, handling, setup and inspection. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1179/1758120614Z.00000000045) |
| **IN06 — Synthetic organic dyes** | Industrial chemistry + chemical feedstocks + process control | Perkin’s mauveine, Britain, **1856**. | New colorant supply chains and dye works; not instant mastery of every color or fiber. [Science and Industry Museum blog](https://blog.scienceandindustrymuseum.org.uk/worlds-first-synthetic-dye/) |
| **IN07 — Industrial chrome tanning** | Chemical supply + controlled wet processing + appropriate equipment | Commercially successful Schultz process, United States, **1884**, as described in contemporary technical literature. | Much shorter tanning stages for suitable products; different properties, inputs and waste streams. [Wikisource](https://en.wikisource.org/wiki/The_New_International_Encyclop%C3%A6dia/Leather) |
| **IN08 — Synthetic polymer fibers** | Polymer chemistry + filament formation + drawing | Nylon 6,6, United States, **1935**. | Synthetic yarn/fabric chains. Keep regenerated cellulose and different polymers as separate recipes. [National Museum of American History](https://americanhistory.si.edu/collections/object/nmah_576) |
| **IN09 — Controlled industrial wood drying** | Drying knowledge + heat/airflow control + suitable buildings | Industrial development across the nineteenth–twentieth centuries; quantified engineering schedules available by the mid-twentieth century. | Drying kilns, scheduled seasoned stock and tighter production tolerances. [US Forest Service R&D](https://research.fs.usda.gov/download/treesearch/32416.pdf) |

For a 150–250-node overall graph, several of these can be **capability families with recipe variants**, rather than separate discoveries for every fiber and tool arrangement. But preserve separate mechanical principles where they create materially different costs or products.

---

# 6. Stylized facts a correct simulation should reproduce

| Pattern | Expected simulation behavior / test |
| --- | --- |
| **Some ordinary preindustrial garments embody hundreds of hours** | A reconstruction-compatible configuration should be able to approach the Lendbreen estimate without artificially assigning hundreds of hours to sewing alone. [Tidsskrift](https://tidsskrift.dk/atr/article/view/167008) |
| **There is no fixed spinner-to-weaver ratio** | Derive staffing from yarn specification, spinning rate, loom speed and fabric construction. Changing one stage changes the required ratio. |
| **Process improvements have diminishing chain-level effects** | In the worked example, a 6.7× spinning improvement yields only a 2.8× garment improvement. |
| **Luxury can be labor-intensive without using much more fiber** | Finer yarn, additional pattern operations and demanding finishing should increase cost independently of garment mass. |
| **Repair and reuse can remain rational even for valuable textiles** | Condition, repair costs and replacement prices should produce long object histories, not annual automatic destruction. Surviving repaired garments and documentary prescriptions support this behavior. [Tidsskrift](https://tidsskrift.dk/atr/article/view/167008) |
| **Manufacturing sophistication predates metal-intensive machinery** | Stone-tool joinery and elaborate hand textiles must remain feasible. “No iron” must not imply crude furniture or plain cloth. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0051374) |
| **Tanneries and timber yards accumulate work in progress** | A long-residence process needs storage and finance even when its active workforce is small. |
| **Mechanization need not erase handmade production** | Small batches, unusual products, inaccessible capital, transport costs and preference can preserve older methods. Museum collections document hand-operated domestic jennies as well as factory machinery. [Smithsonian Institution](https://www.si.edu/object/household-or-domestic-spinning-jenny%3Anmah_1094234) |
| **Greater ownership need not mean proportionately greater use** | Modern-style wardrobe expansion should permit low utilization rather than forcing every item to wear out at an identical annual rate. [Wrap](https://www.wrap.ngo/media-centre/press-releases/nations-wardrobes-hold-16-billion-items-unworn-clothes-people-open-new) |

A useful **internal consistency test**, not a historical employment estimate: if a settlement of 5,000 people requires 250 craft-hours of new textiles per resident annually, that is **1.25 million hours**, or about **833 full-time equivalents at 1,500 productive hours each**. The simulation must find those hours after other obligations. It cannot create them as free household labor.

---

# 7. Recommended agent and institution implementation

## 7.1 Use a small, compositional material model

Avoid hundreds of separate goods named “fine red wool cloth,” “fine blue wool cloth” and so forth.

A cloth batch can instead contain:

```
material_mix
area_m2
mass_kg
yarn_tex_warp / yarn_tex_weft
warp_density / weft_density
structure
color_process / colors
finish
strength / softness / insulation / water_response
quality_variation
provenance
```

A garment references its construction pattern and material batches, then adds:

```
owner
fit
condition
cleanliness
repair_history
style_affiliations
prestige_assessment
worn_hours
```

For leather, substitute area, thickness, tanning/dressing process and defects. For timber, use species, dimensions, moisture, grain/defect grade and processing history.

This makes material quality consequential without requiring a unique inventory class for every combination.

## 7.2 Schedule operations, not animations

Represent production as a job graph with:

`inputs`, `worker_skill`, `equipment`, `active_work_remaining`, `minimum_elapsed_time`, `environmental_conditions`, `batch_capacity`, and `output_distribution`.

A tanning job alternates between active handling and unattended processing. A spinning job advances only when someone works. A loom setup can be shared across several outputs.

The UE5 representation should show meaningful actions—spinning, warping, weaving, scraping, cutting, turning—but the Rust kernel should not simulate individual stitches or fiber contacts.

For 10k–50k people, aggregate repeated work into coarse work sessions and update unattended processes on a slower schedule. Reserve detailed object tracking for visible, owned or economically important goods.

## 7.3 Make specialization emerge from constraints

Support three basic organizational arrangements from the start:

**Household production.** People perform several stages around other obligations. Portable tasks and fixed-equipment tasks have different scheduling opportunities.

**Independent workshops.** Specialists buy inputs or accept customer material, charge for work, and invest in equipment and skill.

**Coordinated production networks.** A merchant, estate, temple, state or other institution supplies materials and distributes stages among producers.

These are proposed institutional templates. Their prevalence should emerge from credit, transport, enforcement, skill distribution and demand—not from an “early era/late era” switch.

Crucially, constrained or coerced labor must still consume real human time. Low monetary payment is not low physical labor cost.

## 7.4 Preserve tacit skill

Use separate competencies for spinning, weaving, dyeing, tailoring, skin processing, shoemaking, joinery, cooperage and turning.

Learning should change:

* throughput;
* material loss;
* defect probability;
* attainable fineness or complexity;
* diagnosis and repair ability.

A general carpenter should not automatically be an expert cooper. Williamsburg’s cooper describes an apprenticeship of roughly **six to seven years**, illustrating the depth of specialist skill even when an expert can make a simple product quickly. [Colonial Williamsburg Podcasts](https://podcast.history.org/2012/01/02/meet-the-cooper/)

Written instructions or a technology transfer should unlock attempts and training, not immediately grant expert execution.

## 7.5 Connect visible dress to actual economic state

For rendering, derive appearance from actual inventory and material properties:

* silhouette from cultural pattern and garment construction;
* drape from material, weight and structure;
* surface from weave, felt, nap or leather finish;
* color from the available dye process and subsequent fading;
* patches and wear from the garment’s history;
* fit from the wearer’s body and alterations.

Do not equate poverty exclusively with brown clothing or wealth exclusively with bright clothing. In TCE’s model, a cheap colored garment, an expensive undyed fine textile and a repaired prestigious garment should all be possible.

## 7.6 What to simplify

Keep **yarn length, batch setup, skill, repair, material compatibility, moisture/processing state and capital tied up in inventory**.

Simplify individual stitches, detailed collagen chemistry, exact microscopic fiber distributions and every species-specific wood defect.

Use a few meaningful recipe variants before adding numerous named historical tools. A distinction that changes only terminology belongs in descriptive content; a distinction that changes labor, inputs, feasible products or institutions belongs in the simulation.

---

# 8. Existing models, games and research resources

**Dwarf Fortress** provides a useful decomposition of thread, cloth, dyeing and garment production, with separate material and workshop concerns. Its principal value here is architectural: intermediate goods create dependencies. Its recipe timing should not be treated as historical calibration. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Textile_industry)

**Vintage Story** makes leatherworking visibly multi-stage, including hide preparation and successive processing treatments. This is a useful presentation pattern for TCE’s work-in-progress system, but its durations and chemistry abstractions are game rules. [Vintage Story Wiki](https://wiki.vintagestory.at/index.php/Leather_working)

**Modern garment line-balancing models** offer task-precedence graphs, worker assignment and bottleneck analysis. Topaloğlu Yıldız and Karabay’s 2022 shirt-production study is an example. TCE does not need to solve a large optimization problem for every household; the useful abstraction is the operation network. [DOI](https://doi.org/10.32710/tekstilvekonfeksiyon.1020866)

**USDA timber-drying models** are directly useful for a species/thickness/weather-dependent process. Simpson and Hart’s *Estimates of Air Drying Times for Several Hardwoods and Softwoods* is a stronger basis than a universal seasoning timer. [US Forest Service R&D](https://research.fs.usda.gov/treesearch/9684)

For calibration work, prioritize the following source groups:

| Research area | Sources / datasets to retain | Main limitation |
| --- | --- | --- |
| Whole textile production chains | Vedeler & Hammarlund, **2017**, Lendbreen reconstruction; Ravnanger and colleagues, **2026**, Borgund reconstruction | Small numbers of products and makers; experimental conditions differ from lifetime historical practice. [Tidsskrift](https://tidsskrift.dk/atr/article/view/167008) |
| Historical spinning productivity | Humphries & Schneider, **2019**, *Spinning the Industrial Revolution*; Allen’s **2020** reply | Disagreement over worker selection, hours, skill and the interpretation of wage/output evidence. Do not pick one universal rate from the debate. [EHS](https://ehs.org.uk/article/spinning-the-industrial-revolution/) |
| Industrial improvement and diffusion | Maw and colleagues, **2022**, *After the Great Inventions* | Historical machinery and productivity estimates are incomplete and product-specific. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13082) |
| Early textile technique | Research on bast-fiber splicing and revised Çatalhöyük fiber identification | Older identifications and assumed production methods sometimes need revision. [Springer](https://link.springer.com/article/10.1007/s12520-018-0677-8) |
| Leather material identification | Elnaggar and colleagues, **2017**, Egyptian leather analysis | Identifies processes on selected objects; does not supply a universal labor budget. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/arcm.12239) |
| Consumption and ownership | Cato; Douglass; BLS historical expenditure series; WRAP wardrobe research | Prescriptions, coerced allowances, expenditure and wardrobe stock must remain separate data types. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Cato/De_Agricultura/B%2A.html) |

The largest unresolved measurement gaps are **representative preindustrial wardrobe turnover by class**, **controlled productivity comparisons across fibers and yarn specifications**, and **comparable active-labor budgets for leather and wooden household goods**. Those gaps justify broad priors and sensitivity tests—not precise-looking universal constants.

**For TCE’s first implementation, preserve the economic distinctions that generate history: fiber-specific routes, yarn-length costs, seasonal labor competition, specialist skill, repair and second-hand circulation, and inventory tied up in long processes. Those mechanisms will produce more believable clothes and household economies than a much larger catalog of items built on fixed crafting times.**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9285b-4354-83ea-b7e2-72e542c15c3c)
