# Visual culture for The Civilization Engine

## Executive recommendation

**Generate visual traditions, not independent random costumes and logos.** For TCE, the strongest design is a system in which materials constrain production, people learn styles through social contact, institutions regulate some symbols, and new designs inherit recognizable features from older ones.

Keep four things separate:

* **Meaning:** what a community associates with a crane, a color, a border, or a garment.
* **Design:** the recognized arrangement of those elements.
* **Rights and expectations:** who may—or must—display it, and on which occasions.
* **Physical object:** the particular embroidered badge, dyed robe, carved seal, or weathered banner.

This separation lets a symbol remain recognizable while its rendering changes, a wealthy person imitate an official without holding office, and a political revolution change approved insignia without instantly replacing every garment.

The evidence supports these mechanisms much more strongly than universal numerical rates of fashion change. Below, **historical measurements and conventions are distinguished from proposed, uncalibrated simulation parameters.**

---

## 1. Mechanisms: implementable causal rules

### 1.1 Visual identity does not require states, agriculture, or social classes

Personal ornament substantially predates farming. Sehasseh and colleagues reported **33 shell beads** from Bizmoune Cave, Morocco, many from deposits dated to **at least 142,000 years ago**. The objects establish ornament use; they do not reveal a recoverable dictionary of clan identities or meanings. [Science](https://www.science.org/doi/10.1126/sciadv.abi8620)

**TCE rule:** permit ornament wherever materials, techniques, social interaction, and discretionary labor allow it. Do not unlock decoration with government, wealth inequality, or a “civilization level.”

Early traditions can be carried by bead dimensions, placement, hairstyle, body decoration, or repeated geometric arrangements—not necessarily large representational emblems. Their meanings should arise through repeated association with people, occasions, and groups.

A useful distinction is between **coordination signals**, which help people recognize affiliation, and **competitive signals**, which distinguish an individual or subgroup. The same ornament can serve both functions.

### 1.2 Different media solve different problems

**Seals authenticate and attribute.** Mesopotamian cylinder seals carried carved designs that produced reversed impressions when rolled on clay. They appear on administrative accounts, receipts, letters, and other documents; surviving examples include seals and impressions from the late fourth millennium BCE. [The Metropolitan Museum of Art](https://www.metmuseum.org/perspectives/cylinder-seals-tiny-treasures-that-leave-a-big-impression)

**TCE rule:** demand for seals should increase with stored property, delegated administration, contracts, and disputes over authorization. Give seals an owner or office, an impression pattern, and a provenance—not merely a decorative texture. A stolen seal can create a false claim of authorization.

**Standards and flags identify and coordinate.** Treat a pole-mounted emblem, military banner, maritime identifier, religious processional object, and national flag as related but distinct instruments. Their historical functions should not be collapsed into “every settlement has a national flag.” Vexillological research distinguishes earlier pole-mounted emblems from cloth flags and documents their varied symbolism. [Flag Institute](https://www.flaginstitute.org/wp/wp-content/uploads/2022/11/ICV27-E5-Morales-Ramirez.pdf)

**TCE rule:** generate these objects when an organization needs visible recognition: a war band must rally, vessels must identify affiliation, or a procession must display corporate identity. A village can possess shared motifs without having an official flag.

**Dress identifies embodied roles.** A garment can communicate occupation, office, wealth, age, ceremony, and affiliation simultaneously. Qing official dress, for example, combined garments, headgear, accessories, and rank badges into a regulated ensemble—not one isolated icon. [University of Alberta](https://www.ualberta.ca/en/the-quad/2025/04/in-photos-birds-and-beasts-wearing-honour-and-order-in-the-qing-dynasty.html)

**TCE rule:** make the outfit a contextual composition. Work, court attendance, mourning, marriage, travel, and military service should select different combinations from the same wardrobe.

### 1.3 Heraldry is one institutional solution, not a universal visual grammar

European heraldry emerged in the **twelfth century** and developed as a system of identification for individuals and corporate bodies. It should be one possible outcome of TCE’s institutions, not the default form of every society’s symbolism. [College of Arms](https://www.college-of-arms.gov.uk/)

A heraldic grammar separates the field, divisions, geometric figures, charges, and tinctures. A familiar convention discourages placing a color on a color or a metal on a metal. The conventional palette contains five principal colors and two metals, alongside other categories such as furs. These are historically specific conventions with exceptions, not universal laws of perception. [The Heraldry Society](https://www.theheraldrysociety.com/wp-content/uploads/2018/03/Historic-Heraldry-Handbook.pdf)

**TCE rule:** let institutions establish optional design protocols:

`permitted components → composition rules → rights of display → inheritance rules → dispute resolution`

A hereditary system might preserve a central charge while adding a distinguishing mark for a branch. Another society might transmit an emblem through an office, a maternal lineage, adoption, initiation, or a craft association. Follow the relevant kinship and institutional rules rather than hard-coding paternal surname inheritance.

Store the **logical description** separately from the drawing. A heraldic “red” need not designate one exact digital shade; the College of Arms explicitly distinguishes blazoned tinctures from exact shades. [College of Arms](https://www.college-of-arms.gov.uk/resources/faqs)

### 1.4 Materials and production shape the available visual vocabulary

Indian textile production demonstrates why color and pattern are production systems rather than arbitrary appearance choices: dyeing can involve mordants, repeated baths, resist processes, block application, weaving, and embroidery. Different fibers and techniques enable different effects. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/indian-textiles-trade-and-production)

**TCE rule:** every visual treatment needs a recipe. A useful cost identity is:

\[
C\_{\text{object}}
=\sum\_m q\_m p\_m+\sum\_s h\_s w\_s
+C\_{\text{transport}}+C\_{\text{waste}}.
\]

Here, \(q\_m\) is material quantity, \(p\_m\) its price, \(h\_s\) skilled labor time, and \(w\_s\) the relevant labor cost. In household production, labor still has an opportunity cost even without cash wages.

Distinguish:

* inexpensive local color from scarce imported color;
* woven pattern from printed imitation;
* fine thread from coarse thread;
* metal, metallic thread, and metal-colored pigment;
* technically difficult ornament from merely large ornament.

**Do not equate saturation with wealth.** Let expense emerge from the actual recipe and supply chain.

Clothing also needs functional properties: insulation, ventilation, coverage, water resistance, durability, and freedom of movement. Arctic clothing provides a clear example of sophisticated layered solutions rather than “primitive clothing.” Museum documentation describes seasonal layering and multiple footwear layers. [Canadian Museum of History](https://www.historymuseum.ca/collections/artifact/42856)

For TCE, climate should influence these performance requirements—not deterministically select one garment silhouette.

### 1.5 Symbol meanings are contextual and historically acquired

A cross-national study by Jonauskaite and colleagues tested **4,598 participants in 30 nations and 22 languages**. It found shared color–emotion associations alongside differences related to linguistic and geographic proximity. These are modern associations with color terms, not evidence for a universal ancient political or ritual color dictionary. [PubMed](https://pubmed.ncbi.nlm.nih.gov/32900287/)

Historical meanings also depend on combinations. In Japanese kimono, cranes can express longevity and good fortune, while red can signify youthful allure. Korean official badges also use cranes, but within a system that communicates bureaucratic rank as well as auspicious meaning. [Victoria and Albert Museum](https://www.vam.ac.uk/articles/kimono)

**TCE rule:** represent meaning as:

`association(symbol, community, context, period, interpretation, strength)`

Do not encode `crane = longevity` or `red = war` globally.

Generate initial associations from locally salient animals, plants, landscapes, occupations, stories, founders, and events. Subsequent use reinforces or changes them. A river bird might first identify a fishing association, later a patron deity, and eventually a political coalition. These are possible simulated histories, not predetermined symbolism.

An institution may declare an official interpretation while ordinary people retain competing interpretations.

### 1.6 Institutions make symbols consequential

Inscribed *tiraz* textiles and robes of honor in Islamic societies could express relationships with rulers and reward service. Their materials, inscriptions, production, distribution, and reuse connected dress to political and economic institutions. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/tiraz-inscribed-textiles-from-the-early-islamic-period)

Similarly, Inka high-status textiles could communicate entitlement and position through restricted designs and exceptional craftsmanship. The meanings of individual *t’oqapu* motifs remain debated; they should not be presented as a securely deciphered alphabet. [Dumbarton Oaks Museum](https://museum.doaks.org/objects-1/info?page=2&query=mfs+any+%22All-T%27oqapu+Tunic%22&sort=9)

**TCE rule:** distinguish five institutional operations:

`grant`, `require`, `reserve`, `prohibit`, and `revoke`.

A grant changes recognized rights. It does not automatically manufacture a garment. A prohibition changes expected sanctions. It does not make imitation physically impossible.

Model compliance through observers, enforcement capacity, occasion, and incentives. An unauthorized badge matters only when someone recognizes it and cares. A badge can also lead an observer to overestimate the wearer’s authority.

Status should therefore be partly **inferred from appearance**, rather than every agent having perfect access to everyone else’s official attributes.

### 1.7 Styles spread through people, objects, and production knowledge

Indian textiles circulated through Asian trade networks before becoming highly fashionable in European markets. This is a useful counterexample to treating Europe as the universal source of fashion innovation. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/indian-textiles-trade-and-production)

Contemporary documentation of Ghanaian kente describes transmission through families, apprenticeships, education, festivals, and other institutions; named designs can refer to proverbs and social situations. [UNESCO ICH](https://ich.unesco.org/en/RL/craftsmanship-of-traditional-woven-textile-kente-02130)

**TCE rule:** maintain separate transmission channels:

**Exposure** teaches someone that a style exists. **Preference transmission** changes whether they admire it. **Technical transmission** enables a workshop to reproduce it. **Trade** supplies finished objects without transferring production knowledge.

This distinction produces useful outcomes: imported prestige goods, local imitations, hybrid designs, and fashions that remain aspirational because people cannot obtain them.

Do not restrict influence to elite-to-commoner copying. Allow peer diffusion, migrant influence, occupational styles, youth differentiation, and rejection of a disliked group’s appearance.

### 1.8 Fashion change and physical replacement run on different clocks

Acerbi, Ghirlanda, and Enquist’s model shows how copying preferences as well as traits can generate fashion cycles without an externally imposed cycle length. Its empirical comparisons include names and dog-breed popularity, so it is a mechanism candidate—not a calibrated model of ancient clothing. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0032541)

**TCE rule:** update desired appearance through social learning, but change actual appearance through dressing, purchase, production, gifts, repair, inheritance, and disposal.

A person can dislike a garment and continue wearing it. A robe can survive its owner. A badge can be replaced while the coat remains. Institutional reform can alter official meanings immediately while physical replacement takes much longer.

Technological change should modify costs and capabilities rather than simply unlock colors. Perkin’s **1856 mauveine** and subsequent industrial production illustrate a change in dye supply and manufacture; they do not mark the invention of colorful clothing. [Science Museum Blog](https://blog.sciencemuseum.org.uk/mauve-mania/)

---

## 2. Parameters and quantitative anchors

### 2.1 Historical measurements and design conventions

These are **case-specific calibration anchors**, not universal parameter estimates.

| Quantity | Value and units | Appropriate use in TCE | Source and confidence |
| --- | --- | --- | --- |
| Principal conventional heraldic palette | **5 colors + 2 metals**; categorical count | One historically grounded heraldic grammar; retain exceptions and additional categories | Heraldry Society. **High for convention; not universal.** [The Heraldry Society](https://www.theheraldrysociety.com/wp-content/uploads/2018/03/Historic-Heraldry-Handbook.pdf) |
| Recommended simple flag palette | **2–3 colors per flag** | Optional distant-recognition design objective, not a historical restriction | NAVA, *Good Flag, Bad Flag*. **High as stated advice; not empirical law.** [Nava](https://www.nava.org/good-flag-bad-flag) |
| Qing official hierarchy | **9 civil + 9 military ranks** | Example of mapping office categories and rank to regulated motifs | University of Alberta Museums. **High.** [University of Alberta](https://www.ualberta.ca/en/the-quad/2025/04/in-photos-birds-and-beasts-wearing-honour-and-order-in-the-qing-dynasty.html) |
| Qing badge arrangement and scale | **2 badges**, front and back; approximately **30 × 30 cm** | Reference dimensions and placement for this particular costume system | Same museum study. **High for arrangement; approximate dimensions.** [University of Alberta](https://www.ualberta.ca/en/the-quad/2025/04/in-photos-birds-and-beasts-wearing-honour-and-order-in-the-qing-dynasty.html) |
| Cylinder-seal scale | Illustrated examples approximately **1.9–3.6 cm high** | Useful miniature-object asset range; not a population distribution of all seals | Met collection examples. **High for the objects; low generalizability.** [The Metropolitan Museum of Art](https://www.metmuseum.org/perspectives/cylinder-seals-tiny-treasures-that-leave-a-big-impression) |
| Ostrich-eggshell bead diameter | Eastern sample **6.9 ± 1.2 mm**; later southern sample **4.5 ± 0.9 mm**, reported mean ± SD | Test stable regional preferences and small but socially meaningful differences; these samples cover different temporal scopes | Miller & Wang. **High measurement confidence; interpretation more uncertain.** [Nature](https://www.nature.com/articles/s41586-021-04227-2) |
| Exceptional Inka textile fineness | **98–108 wefts/cm** in the all-*t’oqapu* tunic | Demonstrate that fine workmanship, not merely bright color, can distinguish elite production | Dumbarton Oaks object study. **High for this exceptional artifact.** [Dumbarton Oaks Museum](https://museum.doaks.org/objects-1/info?page=2&query=mfs+any+%22All-T%27oqapu+Tunic%22&sort=9) |
| Periodic ornamental symmetry classes | **7 frieze groups; 17 wallpaper groups** | Mathematical library for border and surface-pattern generation | Liu et al., computational pattern research. **Exact mathematical classification under its assumptions.** [PubMed](https://pubmed.ncbi.nlm.nih.gov/15376882/) |
| Animal imagery in one flag corpus | **961/4,804 flags ≈ 20%** | Benchmark only for the sampled North American subnational flags, including animals inside seals and shields | Morales Ramirez; November 2016 corpus. **Moderate: catalog coverage and selection matter.** [Flag Institute](https://www.flaginstitute.org/wp/wp-content/uploads/2022/11/ICV27-E5-Morales-Ramirez.pdf) |

The symmetry counts classify ideal periodic patterns, not the number of cultural styles or levels of artistic development. Likewise, the flag percentage should not become a global animal-emblem probability.

### 2.2 Proposed starting parameters

The following values are **engineering starting points and sensitivity-test ranges**. They are not measurements of historical populations.

| Parameter | Starting value; test range | Units / application | Source and confidence |
| --- | --- | --- | --- |
| Preference-update interval | **30; 7–90** | Simulated days; stagger agents across updates | TCE proposal; historical calibration absent |
| Sampled social observations | **8; 4–16** | Observations per agent per preference update | Computational approximation; not a limit on real social networks |
| Preference learning step | **0.2; 0.05–0.5** | Fraction of difference between existing and observed preference incorporated per update | TCE proposal; low empirical confidence |
| Frequency-dependence exponent \(\gamma\) | **1.2; 0.5–2.0** | Dimensionless; defined in the choice model below | Exploratory model parameter |
| Prestige weighting | **2; 1–5** | Relative influence of a high-prestige versus matched ordinary exemplar | Exploratory; normalize other influences |
| Novel variant probability | **0.001; 0.0001–0.01** | Per eligible style-choice or design event—not per daily agent tick | TCE proposal; sweep logarithmically |
| Feature retention in a derived design | **0.9; 0.7–1.0** | Probability per mutable inherited feature | TCE proposal; legally invariant features remain fixed |
| Candidate designs evaluated | **16; 8–32** | Candidates per new commission | Computational quality/cost control |
| Garment replacement fallback | **0.3; 0.1–1.0** | Events per item-year, only when no detailed wear/repair model exists | Placeholder, not historical clothing-life estimate |

For any rate \(\lambda\), convert to a timestep probability using:

\[
p(\Delta t)=1-e^{-\lambda\Delta t}.
\]

The replacement fallback should ultimately be superseded by condition, repairability, use intensity, household resources, and production costs. Otherwise, an arbitrary wardrobe lifetime will quietly determine the apparent speed of cultural change.

Do not independently tune “fashion speed” until material replacement and social exposure are behaving plausibly.

---

## 3. Variation across eras and regions

### 3.1 Historical settings—not progression stages

| Setting | Evidence or characteristic possibility | Modeling implication |
| --- | --- | --- |
| **Foragers** | Ornament and geographically extensive stylistic connections long predate farming. [University of Arizona](https://experts.arizona.edu/en/publications/early-middle-stone-age-personal-ornaments-from-bizmoune-cave-essa/) | Permit elaborate identities without cities, literacy, or hereditary classes. Portability and access matter more than an era label. |
| **Early farming and later early urban systems** | Increasing storage and administration can make ownership and authorization marks useful; late-fourth-millennium Mesopotamian seals provide concrete examples. [The Metropolitan Museum of Art](https://www.metmuseum.org/perspectives/cylinder-seals-tiny-treasures-that-leave-a-big-impression) | Generate administrative identities when delegation and property systems demand them—not automatically when farming begins. |
| **Pre-industrial states and commercial networks** | Rank-regulated dress, patronage textiles, hereditary identifiers, and long-distance textile trade coexist. [University of Alberta](https://www.ualberta.ca/en/the-quad/2025/04/in-photos-birds-and-beasts-wearing-honour-and-order-in-the-qing-dynasty.html) | Distinguish court, household, occupational, religious, and regional traditions. There need not be one national costume. |
| **Industrializing societies** | Synthetic dye production can expand supply; new technologies can also enter older decorative vocabularies, as with vehicles on twentieth-century kimono. [Science Museum Blog](https://blog.sciencemuseum.org.uk/mauve-mania/) | Change supply chains and channels of exposure while allowing established garment forms to persist. |
| **Modern societies** | Commercial fashion, public symbols, and deliberately maintained craft identities coexist; kente remains an evolving means of identity construction. [Multimedia Laboratory](https://mmlab.ie.cuhk.edu.hk/projects/DeepFashion.html) | High connectivity need not erase local identities. Model specialist, ceremonial, and heritage demand alongside mass production. |

### 3.2 Regional cases that require different grammars

| Region or tradition | Important distinction | Procedural consequence |
| --- | --- | --- |
| **Mesopotamian seals** | Small carved scenes operate through impressions and administrative use | Generate legible relief compositions, owner attribution, and mirrored impressions—not colored miniature flags. [The Metropolitan Museum of Art](https://www.metmuseum.org/perspectives/cylinder-seals-tiny-treasures-that-leave-a-big-impression) |
| **European heraldry** | Identity can be encoded in a structured, reproducible description and transmitted through regulated rights | Use a composition grammar with institutional inheritance and differentiation rules. [College of Arms](https://www.college-of-arms.gov.uk/) |
| **Chinese official dress** | Birds and beasts identify different official hierarchies within an ensemble | Attach motifs to offices and protocol; wealth alone should not confer the same recognized entitlement. [University of Alberta](https://www.ualberta.ca/en/the-quad/2025/04/in-photos-birds-and-beasts-wearing-honour-and-order-in-the-qing-dynasty.html) |
| **Korean rank badges** | An adopted system was revised: motifs were standardized in **1871**, and the system suspended in **1899** | Version institutional rules over time rather than treating a culture’s symbolism as immutable. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/50672) |
| **Japanese kimono and crests** | Surface pattern, material, accessories, and seasonal treatment can vary substantially within a relatively stable garment form; family crests also occur on objects beyond clothing | Separate silhouette, surface pattern, family identifier, and occasion. [Victoria and Albert Museum](https://www.vam.ac.uk/articles/kimono) |
| **Islamic tiraz and robes of honor** | Inscriptions, patronage, and distribution communicate political relationships | Permit script-bearing and non-shield identities; connect garments to grants and diplomatic exchange. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/tiraz-inscribed-textiles-from-the-early-islamic-period) |
| **Ghanaian kente** | Strip composition, named designs, social context, and specialist transmission matter | Generate meaningful pattern families assembled through a particular production grammar, not generic “African prints.” [UNESCO ICH](https://ich.unesco.org/en/RL/craftsmanship-of-traditional-woven-textile-kente-02130) |
| **Andean/Inka textiles** | Geometric composition and extraordinary weaving skill can carry status | Let technical fineness and controlled motif combinations compete with jewelry or heraldic display as prestige channels. [Dumbarton Oaks Museum](https://museum.doaks.org/objects-1/info?page=2&query=mfs+any+%22All-T%27oqapu+Tunic%22&sort=9) |
| **Pacific and other barkcloth traditions** | Much barkcloth is beaten rather than woven; materials and production differ across regions | Provide separate manufacture and ornament systems. Do not require every textile-like object to pass through a loom. [National Museums Scotland](https://www.nms.ac.uk/discover-catalogue/the-ancient-craft-of-barkcloth-across-the-world) |
| **Arctic clothing traditions** | Functional layering and sophisticated skinworking coexist with cultural distinctions | Generate climate-appropriate constructions first, then vary culturally meaningful details within them. [Canadian Museum of History](https://www.historymuseum.ca/collections/artifact/42856) |

These should inform **authored construction families and social mechanisms**, not become indivisible ethnic asset bundles. TCE should be able to produce a society with one region’s structural clothing solution, another learned dye process, and an independently developed system of rank markers.

---

## 4. Stylized facts and validation targets

### Identity should precede centralized government

A simulation that produces no meaningful ornament before cities would contradict the archaeological evidence. The Bizmoune finds alone place personal ornament far earlier than agriculture. Validate that small, non-state populations develop persistent but variable visual conventions. [University of Arizona](https://experts.arizona.edu/en/publications/early-middle-stone-age-personal-ornaments-from-bizmoune-cave-essa/)

### Contact should explain resemblance better than mere proximity

Miller and Wang analyzed **1,516 ostrich-eggshell beads from 31 sites** and inferred an approximately **3,000-km stylistic connection around 50–33 thousand years ago**. This is an inference about networks, not proof of one ethnicity or direct end-to-end travel. [Nature](https://www.nature.com/articles/s41586-021-04227-2)

For TCE, compare stylistic similarity against interaction-network distance, migration, and trade—not only kilometers. Connected distant settlements should sometimes resemble one another more than neighboring but socially separated groups.

### Cultural coherence should coexist with internal differentiation

Kimono provides a useful case in which recognizable construction coexists with distinctions in material, ornament, and occasion. [Victoria and Albert Museum](https://www.vam.ac.uk/articles/kimono)

A suitable test is therefore not “everyone in culture A looks alike.” It is whether viewers can recognize related design principles while also distinguishing work clothes, ceremonial dress, wealth, and subgroups.

### Office, wealth, and inherited identity should remain distinguishable

Qing rank insignia and Islamic robes of honor illustrate institutional pathways to distinctive appearance. [University of Alberta](https://www.ualberta.ca/en/the-quad/2025/04/in-photos-birds-and-beasts-wearing-honour-and-order-in-the-qing-dynasty.html)

Test whether a wealthy merchant, impoverished noble, appointed official, and decorated servant can look different for different reasons. Giving the same increasingly ornate outfit to every high-status agent loses the mechanism.

### Old forms should survive new materials and meanings

Twentieth-century kimono could incorporate cars, trains, aircraft, and increasingly militaristic imagery without abandoning the garment category. [Victoria and Albert Museum](https://www.vam.ac.uk/articles/kimono)

A correct generator should allow **surface innovation without compulsory silhouette replacement**. Conversely, an old emblem should sometimes survive a change in garment construction.

### Fashion should produce unequal popularity and irregular turnover

Acerbi and colleagues examined strongly unequal popularity and relationships between adoption and abandonment in cultural time series. These patterns support testing more than neutral random replacement, but their datasets do not establish a universal clothing-cycle period. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0032541)

Measure variant popularity distributions, adoption curves, persistence, and cohort differences. Do not enforce a fixed twenty-year cycle or make every successful fashion spread across the whole world.

### Legal redesign should not erase physical history

Korea’s changing badge rules provide a concrete example of institutional discontinuity. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/50672)

In TCE, distinguish the date a rule changes from the distribution of objects in use. Track newly compliant production, retained old garments, deliberate resistance, and obsolete objects in storage. The lag itself is a model output; the evidence here does not provide a universal replacement lag.

---

## 5. Recommended implementation

### 5.1 A compact data model

Use shared definitions with individual references, rather than storing a complete procedural description on every person.

| Record | Essential contents |
| --- | --- |
| **Visual tradition** | Weighted preferences for forms, motifs, palettes, ornament placement, and construction families; links to contributing traditions |
| **Symbol** | Semantic identity; alternative visual realizations; associations by community and context |
| **Design** | Structured composition; grammar version; parent designs; creator; commissioning event; deterministic seed |
| **Display right** | Holder or qualifying role; required/permitted contexts; issuing institution; inheritance and revocation rules |
| **Garment or ornament recipe** | Materials, processes, labor, construction family, functional properties, compatible decoration slots |
| **Physical item** | Owner, recipe, design, production batch, quality, condition, alterations, provenance |
| **Individual preferences** | Affinities, admired exemplars, conformity or distinction tendency, occasion-specific preferences |
| **Institutional dress protocol** | Required ensemble, prohibited uses, rank mappings, inspection and sanctions |

A person can participate in household, occupational, religious, neighborhood, cohort, and political traditions simultaneously. Avoid deriving all appearance from one culture ID.

### 5.2 A tractable choice model

At a dressing or acquisition opportunity, evaluate a bounded set of feasible options:

\[
P\_i(v)=
\frac{F\_i(v)\exp[U\_i(v)]}
{\sum\_{u} F\_i(u)\exp[U\_i(u)]},
\]

with

\[
U\_i(v)=
\beta\_p\,\mathrm{Prestige}\_i(v)
+\gamma\ln(f\_i(v)+\epsilon)
+\beta\_h\,\mathrm{Affinity}\_i(v)
+\beta\_n\,\mathrm{Novelty}\_i(v)
-\beta\_c\,\mathrm{Cost}\_i(v)
-\beta\_s\,\mathrm{Sanction}\_i(v)
-\beta\_t\,\mathrm{Mismatch}\_i(v).
\]

This is a **proposed TCE model**, not an estimated historical equation.

Here, \(F\_i(v)\) expresses physical availability and genuine feasibility; \(f\_i(v)\) is frequency among relevant observed exemplars. Normalize costs, sanctions, and functional mismatch before combining them.

Use legal restrictions primarily in the sanction term, rather than automatically setting feasibility to zero. Otherwise, imitation and defiance become impossible.

When no new option is feasible, retain an existing item or select an available fallback. Preference updating must still be possible without acquisition.

### 5.3 Generate designs through constrained composition

For each new commission:

**Select purpose and medium.** Authentication, battlefield recognition, court rank, ordinary dress, ceremony, or decoration imply different constraints.

**Retrieve inherited vocabulary.** Draw candidates from the commissioner’s affiliations, local workshops, existing designs, and known imports.

**Choose meaningful components.** Weight motifs by actual relationships and remembered events. A settlement’s granary association is a better reason for a grain motif than an arbitrary global “agricultural civilization” label.

**Compose using an appropriate grammar.**

* Heraldic systems: fields, divisions, charges, and associated rights.
* Flags: fields, broad partitions, devices, and optional inscriptions.
* Seals: impression-oriented scenes, signs, names, and borders.
* Clothing: compatible construction families, fabrics, trims, and accessories.
* Ornament: motif repetition, border grammars, symmetry operations, and controlled irregularity.

**Evaluate candidates.** Score recognition, similarity to ancestors, local ambiguity, manufacture cost, protocol compliance, and the commissioner’s goals.

**Record provenance.** Save why the design exists and what it derives from.

NAVA’s advice—simplicity, meaningful symbolism, limited contrasting colors, and distinctiveness or deliberate relatedness—is useful for one flag-design objective. It should not reject every historically plausible complex flag or inscription-bearing banner. [Nava](https://www.nava.org/good-flag-bad-flag)

Do not require global uniqueness. Identical or confusingly similar signs are plausible when groups have little contact. Ambiguity should become costly primarily within an actual recognition network.

### 5.4 Let changes have specific causes

Useful design events include institutional foundation, accession, marriage alliance, lineage branching, migration, workshop innovation, religious change, conquest, coalition formation, and political repudiation.

Each event should modify particular components rather than rerolling the entire design. A branch might retain the main motif and change a border. A coalition might combine identifiers. A successor might retain an older symbol to claim continuity—or remove it to reject that claim.

These transformations should be **grammar-dependent**. European-style quartering is not the generic representation of every marriage or political union.

### 5.5 Rust simulation and Unreal rendering

Keep the Rust-side state semantic and deterministic. Send Unreal compact appearance references and changes, not instructions to independently invent a new identity.

For a first implementation:

* Use authored garment construction families with bounded variations.
* Share meshes, motif assets, and cached pattern textures.
* Represent dye batches, wear, repairs, and quality as variations of physical items.
* Rebuild appearance after relevant events, not every simulation tick.

At **50,000 agents and eight sampled observations per monthly update**, the social-learning pass contains approximately **400,000 observation evaluations per simulated month**. That is an operation count, not a performance benchmark; actual cost depends on candidate evaluation and data access.

Crucially, **simulation visibility must not depend on the player’s camera**. An agent’s ability to notice a badge should depend on simulated proximity, attention, and context. Rendering distance and texture detail are separate optimizations.

Reserve detailed cloth simulation and fine ornament rendering for situations where they contribute visually. The economic and social consequences of an object should remain active even when its geometry is simplified.

### 5.6 Existing systems worth studying

| System | Useful lesson | Limitation |
| --- | --- | --- |
| **Armoria / Azgaar** | Procedural heraldry generation and editing; reusable separation of design data and rendered output | It generates heraldry, not a complete social history of symbols. Its code is MIT, but some complex charges are restricted to non-commercial use. Audit assets individually. [GitHub](https://github.com/Azgaar/Armoria) |
| **DrawShield** | Rendering from blazon demonstrates the value of a formal description beneath multiple visual realizations | Its grammar is heraldic, not a universal system for dress and ornament. [DrawShield](https://drawshield.net/) |
| **Crusader Kings III** | A practical game example of editable dynastic coats of arms | Treat this as an interface and identity-management reference, not evidence for historical diffusion rates. Official update notes document the designer and custom dynastic arms. [Steam Store](https://store.steampowered.com/news/posts/?enddate=1647954248&feed=steam_community_announcements) |
| **Acerbi–Ghirlanda–Enquist fashion model** | Preferences themselves can be transmitted, allowing endogenous turnover | Add material availability, institutions, and item persistence; do not transfer its empirical proxy datasets directly into ancient clothing parameters. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0032541) |

**Recommended v1 scope:** coherent traditions, item-based clothing, institutional insignia, inherited design variation, and limited network diffusion. Defer arbitrary garment topology, unrestricted image generation, and a fully general symbolic language.

---

## 6. Sources, datasets, and evidence limits

### High-value datasets and research resources

| Resource | Practical use | Important limitation |
| --- | --- | --- |
| **Miller & Wang, 2022, *Nature*, bead dataset and supplements** | Dated measurements for testing geographic variation, continuity, and contact effects | Archaeological assemblages are not direct observations of individual preferences or ethnic boundaries. [Nature](https://www.nature.com/articles/s41586-021-04227-2) |
| **Metropolitan Museum Open Access API** | Object dates, materials, dimensions, descriptions, and eligible images for building an annotated reference corpus | Filter by object-level rights and preserve uncertainty in dating and attribution. Public-domain/open-access eligibility is not identical for every object. [Met Museum API](https://metmuseum.github.io/) |
| **Dumbarton Oaks textile catalog** | Detailed construction and technical descriptions, including the all-*t’oqapu* tunic | Exceptional objects are not representative household wardrobes; semantic interpretation can remain contested. [Dumbarton Oaks Museum](https://museum.doaks.org/objects-1/info?page=2&query=mfs+any+%22All-T%27oqapu+Tunic%22&sort=9) |
| **DeepFashion, Liu et al., CVPR 2016** | Modern clothing taxonomy and recognition research: over **800,000 images**, **50 categories**, and **1,000 attributes** | Modern commercial-image bias; explicitly **non-commercial research only**, including restrictions on derived data. Not a ready-made commercial TCE asset source. [Multimedia Laboratory](https://mmlab.ie.cuhk.edu.hk/projects/DeepFashion.html) |
| **Morales Ramirez’s subnational-flag corpus** | Motif coding and a reproducible example of counting devices inside versus outside seals and shields | A geographically and temporally bounded catalog, not a universal sample of flags. [Flag Institute](https://www.flaginstitute.org/wp/wp-content/uploads/2022/11/ICV27-E5-Morales-Ramirez.pdf) |

The principal scholarly anchors are Sehasseh et al. on early ornament, Miller and Wang on prehistoric stylistic networks, Jonauskaite et al. on color associations, and Acerbi et al. on fashion dynamics. Museum object research supplies much of the strongest evidence for construction, materials, dimensions, and institutional use. These answer different questions and should not be treated as interchangeable evidence. [Science](https://www.science.org/doi/10.1126/sciadv.abi8620)

### What remains uncertain

**Meaning is often less secure than form.** An artifact can establish a repeated motif without establishing its meaning. The *t’oqapu* case is especially important: competing interpretations should not become an authoritative procedural dictionary. [Dumbarton Oaks Museum](https://museum.doaks.org/objects-1/info?page=2&query=mfs+any+%22All-T%27oqapu+Tunic%22&sort=9)

**Modern psychology does not directly calibrate ancient symbolism.** Color–emotion experiments help reject overly simple universal assumptions, but do not specify the meanings of ancient wedding dress, royal insignia, or religious colors. [PubMed](https://pubmed.ncbi.nlm.nih.gov/32900287/)

**Design advice is not historical behavior.** A modern flag-design guide offers objectives that a designer may prefer; real institutions may prioritize inherited legitimacy, textual specificity, or coalition compromise instead. [Nava](https://nava.org/content.aspx?club_id=622278&module_id=475717&page_id=22)

**Transportable annual rates remain thin.** The sources assembled here do not establish universal rates for pre-industrial fashion adoption, garment replacement, motif innovation, or compliance with dress rules. Those parameters require sensitivity analysis and calibration against specific regional sequences.

For TCE, the central success criterion is therefore not the number of attractive combinations. It is whether an appearance has an intelligible history: **someone made it with available skills and materials; someone learned to value it; someone acquired the right or incentive to display it; and later people preserved, copied, altered, or rejected it.**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9283a-30ac-83e9-9fcf-a55b28d25f6f)
