# Services and Care

This guide describes M10c as implemented in the integrated `main` tree on 2026-10-24. M10c is underway: slices DZ (care beyond the household) and EA (birth care) are complete. Teaching, literacy, inoculation and a health office remain ahead. See [the M10 plan entry](../../PROJECT_PLAN.md#7-milestones) and [ADR-0035](../../decisions/0035-services-letters-and-notes.md). The research guide recommends modeling access to specific services and separating practical competence from theory ([research 05-05](../../research/05-demography-health/05-medicine-healing.md)); current implementation is only an initial, limited step.

## Care beyond the household

Close kin who live elsewhere can hear that a home is sick and travel to tend its residents, sometimes bringing the ill person to their own home. When a household has more ill residents than its healthy members can tend, it may call kin, who ask no payment, or someone it knows, who receives a food fee for each day of care. A household's observed outcomes of care shape its reputation and whom it asks next. The observer's person page records care given and its result.

The kernel owns illness, travel, care capacity, payment and remembered outcomes. The observer presents these records and does not decide who recovers or who is called.

## Birth care

The first EA slice divides maternal deaths in childbirth among bleeding, fever and obstructed labor. The authored techniques `birth_attending` and `clean_hands` can reduce modeled causes when a knowledgeable attendant is present. An attendant who has recently tended the ill without clean hands makes fever more likely. A household with a woman due within a week and no nearby household member or close kin who knows birth care can call a known household whose member has attended a birth. Kin ask no fee; other attendants can be paid a food fee at the birth. The call and its outcome are recorded, and the observer shows births attended. Later M10c teaching, literacy, inoculation and health-office slices are not complete.

These mechanisms are deliberately limited. The code and selected checks do not establish that mortality rates or care practices reproduce any specific historical population.

## Evidence and status

Save schema 191, content API 179 and wire schema 1.104 include the current M10 work. There is no committed M10c demo. The latest actual observer video is M9; see [the gallery](../../demos/README.md) and [persistence and interfaces](persistence-and-interfaces.md).
