//! Cement, steel and rolling (M11e, first part; ADR-0037 §2): the chain's recipes sit in research
//! 11-14's envelopes, nobody starts knowing it, and a works blows pig to steel and rolls it into
//! sections only with each machine's crew at it. A steel skeleton (M11e, second part; ADR-0037
//! §3-§4): a workshop framed in sections is raised of them and its members are checked as steel,
//! and builders who do not know it design as before. Fire on bare steel (M11e, third part;
//! ADR-0037 §5): burnt out, a skeleton's steel keeps 0.4 of its strength, and timber chars as
//! before.

use std::path::Path;
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim};

fn content() -> &'static ContentRegistry {
    static CONTENT: OnceLock<ContentRegistry> = OnceLock::new();
    CONTENT.get_or_init(|| {
        let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("content/ is above the crate");
        civ_content::load(&root)
            .registry
            .expect("the core content loads")
    })
}

fn world(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Steel".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

fn good(sim: &Sim, id: &str) -> usize {
    let id = format!("core:good/{id}");
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect("a good")
}

fn recipe(sim: &Sim, id: &str) -> civ_agents::params::RecipeDef {
    let id = format!("core:recipe/{id}");
    sim.rules()
        .catalog
        .recipes
        .iter()
        .find(|r| r.id == id)
        .cloned()
        .expect("a recipe")
}

fn activity(sim: &Sim, id: &str) -> usize {
    let id = format!("core:activity/{id}");
    sim.rules()
        .catalog
        .activities
        .iter()
        .position(|a| a.id == id)
        .expect("an activity")
}

fn technique(sim: &Sim, id: &str) -> usize {
    let id = format!("core:technique/{id}");
    sim.rules()
        .catalog
        .techniques
        .iter()
        .position(|t| t.id == id)
        .expect("a technique")
}

/// Kilograms of `g` a unit of `r` takes.
fn takes(r: &[(usize, f64)], g: usize) -> f64 {
    r.iter().filter(|&&(i, _)| i == g).map(|&(_, kg)| kg).sum()
}

/// Grown people, with their households, oldest households first.
fn grown(sim: &Sim) -> Vec<(PermanentId, PermanentId)> {
    let now = sim.now();
    let mut out: Vec<(PermanentId, PermanentId)> = sim
        .people()
        .people
        .iter()
        .filter(|(_, p)| p.age_years(now) >= 20.0)
        .map(|(_, p)| (p.household, p.id))
        .collect();
    out.sort();
    out
}

/// Sets a household's stores of the goods given by content id.
fn set(sim: &mut Sim, household: PermanentId, kg: &[(&str, f64)]) {
    let goods = sim.rules().catalog.goods.clone();
    let h = sim
        .people_mut_for_tests()
        .households
        .iter_mut()
        .map(|(_, h)| h)
        .find(|h| h.id == household)
        .expect("household");
    for &(id, v) in kg {
        let g = goods
            .iter()
            .position(|d| d.id == format!("core:good/{id}"))
            .expect(id);
        h.stores[g] = v;
    }
}

#[test]
fn the_chain_from_limestone_and_pig_sits_in_the_research_envelopes() {
    let sim = world(3);
    let [stone, clay, coal, clinker, pig, steel] = [
        "stone",
        "clay",
        "coal",
        "cement_clinker",
        "pig_iron",
        "steel",
    ]
    .map(|g| good(&sim, g));
    let burn = recipe(&sim, "burn_clinker");
    // Research 11-14 §2.2: about 1.7 t of raw material a tonne of clinker.
    let raw = takes(&burn.inputs, stone) + takes(&burn.inputs, clay);
    assert!((1.5..=1.9).contains(&raw), "{raw} kg of raw a kilogram");
    // And 3.0-6.5 GJ a tonne of clinker, here coal at 28 MJ a kilogram over a full kiln.
    let gj_t = takes(&burn.session_inputs, coal) / burn.max_units * 28.0;
    assert!((3.0..=6.5).contains(&gj_t), "{gj_t} GJ a tonne");
    // Clinker is a burn that runs for days; grinding it is powered and at once.
    assert!(burn.elapsed_days >= 1.0);
    let grind = recipe(&sim, "grind_cement");
    assert!(takes(&grind.inputs, clinker) > 0.0 && grind.kwh_per_unit > 0.0);
    assert_eq!(grind.elapsed_days, 0.0);
    // Steel comes from pig, and sections and bar from steel, by a shaft's work.
    let blow = recipe(&sim, "make_steel");
    assert!(takes(&blow.inputs, pig) >= 1.0 && blow.kwh_per_unit > 0.0);
    for r in ["roll_sections", "roll_bar"] {
        let roll = recipe(&sim, r);
        assert!(
            takes(&roll.inputs, steel) >= 1.0 && roll.kwh_per_unit > 0.0,
            "{r}"
        );
    }
}

#[test]
fn nobody_starts_knowing_cement_steel_or_rolling_and_each_stands_on_the_works_before_it() {
    let sim = world(5);
    let [cement, steelmaking, rolling] =
        ["portland_cement", "steelmaking", "rolling"].map(|t| technique(&sim, t));
    for (_, p) in sim.people().people.iter() {
        for t in [cement, steelmaking, rolling] {
            assert!(!p.knows(t), "a founder knows technique {t}");
        }
    }
    let techniques = &sim.rules().catalog.techniques;
    let requires = |t: usize, on: &str| {
        let on = technique(&sim, on);
        techniques[t]
            .requires
            .iter()
            .all(|route| route.contains(&on))
    };
    assert!(requires(cement, "lime_burning"));
    assert!(requires(steelmaking, "blast_furnace"));
    assert!(requires(rolling, "steelmaking"));
}

#[test]
fn a_works_blows_pig_to_steel_and_rolls_sections_only_with_each_machines_crew() {
    let mut sim = world(3);
    sim.advance_minutes(24 * 60).expect("advances");
    let [steel, section, converter, mill, wheel, pig, coal] = [
        "steel",
        "steel_section",
        "converter",
        "rolling_mill",
        "water_wheel",
        "pig_iron",
        "coal",
    ]
    .map(|g| good(&sim, g));
    let (owner, who) = grown(&sim)[0];
    set(
        &mut sim,
        owner,
        &[
            ("pig_iron", 400.0),
            ("coal", 400.0),
            ("water_wheel", 1.0),
            ("converter", 1.0),
            ("rolling_mill", 1.0),
            ("grain", 5000.0),
        ],
    );
    let firm = sim
        .with_ctx_for_tests(|pop, ctx| pop.found_firm_for_tests(ctx, owner, who, steel))
        .expect("a works");
    for tool in [wheel, converter, mill] {
        assert!(sim.with_ctx_for_tests(|pop, ctx| pop.put_in_station(ctx, firm, tool, 1.0)));
    }
    sim.land_mut_for_tests()
        .installations
        .push(civ_land::power::Installation {
            id: PermanentId::from_raw(9_000_003).expect("id"),
            reach: 0,
            at: [0.0, 0.0],
            head_m: 2.0,
            race_m3s: 0.5,
            efficiency: 0.3,
            rated_kw: 3.0,
            owner: firm,
            building: None,
        });
    sim.land_mut_for_tests().reach_discharges[0] = 5.0;
    sim.people_mut_for_tests()
        .firms
        .iter_mut()
        .find(|f| f.id == firm)
        .expect("works")
        .add_line(section as u16);
    let (blow, roll) = (
        activity(&sim, "make_steel"),
        activity(&sim, "roll_sections"),
    );
    let hands: Vec<PermanentId> = grown(&sim)
        .into_iter()
        .map(|(_, p)| p)
        .filter(|&p| p != who)
        .take(3)
        .collect();
    assert_eq!(hands.len(), 3, "three more hands");
    let start = sim.now();
    let held = |sim: &Sim, g: usize| sim.people().firm(firm).expect("works").stores[g];
    let owned = |sim: &Sim, g: usize| sim.people().household(owner).expect("owner").stores[g];
    let made = |sim: &mut Sim, act: usize, g: usize| {
        let before = held(sim, g);
        sim.with_ctx_for_tests(|pop, ctx| pop.firm_make_for_tests(ctx, who, act, 120, firm));
        held(sim, g) - before
    };
    // Two at the converter are one too few; the third blows it.
    sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(who, blow, firm, start));
    sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(hands[0], blow, firm, start));
    assert_eq!(
        made(&mut sim, blow, steel),
        0.0,
        "two hands idle a converter of three"
    );
    sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(hands[1], blow, firm, start));
    let pig_before = owned(&sim, pig);
    let blown = made(&mut sim, blow, steel);
    assert!(blown > 0.0, "three hands blow steel: {blown}");
    assert!(
        (pig_before - owned(&sim, pig) - 1.15 * blown).abs() < 1e-6,
        "1.15 kg of pig a kilogram of steel"
    );
    // Three at the mill are one too few; the fourth rolls the works' own steel.
    for &h in std::iter::once(&who).chain(&hands[..2]) {
        sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(h, roll, firm, start));
    }
    assert_eq!(
        made(&mut sim, roll, section),
        0.0,
        "three hands idle a mill of four"
    );
    sim.with_ctx_for_tests(|pop, _| pop.at_work_for_tests(hands[2], roll, firm, start));
    let (steel_before, coal_before) = (held(&sim, steel), owned(&sim, coal));
    let rolled = made(&mut sim, roll, section);
    assert!(rolled > 0.0, "four hands roll sections: {rolled}");
    assert!(
        (steel_before - held(&sim, steel) - 1.05 * rolled).abs() < 1e-6,
        "1.05 kg of steel a kilogram of section"
    );
    assert!(
        (coal_before - owned(&sim, coal) - 0.3 * rolled).abs() < 1e-6,
        "0.3 kg of coal to reheat a kilogram"
    );
}

/// The first household and its home.
fn first_household(sim: &Sim) -> (PermanentId, (f32, f32)) {
    let (_, h) = sim
        .people()
        .households
        .iter()
        .min_by_key(|(_, h)| h.id)
        .expect("a household");
    (h.id, h.home)
}

/// The workshop of two bays with a loft its builders design, and the same as a steel skeleton
/// (frame version 4: post system 2, steel sections in the fifth slot).
fn workshops(sim: &Sim, at: (f32, f32)) -> (civ_grammar::BuildingSpec, civ_grammar::BuildingSpec) {
    use civ_grammar::frame_params as fp;
    let catalog = &sim.rules().catalog;
    let def = workshop(sim);
    let shape = civ_agents::build::Shape::Bays {
        bays: 2,
        storeys: 1,
        lofts: 1,
    };
    let timber =
        civ_agents::build::design_shape(def, &catalog.goods, shape, at, None).expect("a workshop");
    let mut steel = timber.clone();
    steel.version = 4;
    steel.params[fp::POST_SYSTEM] = 2;
    steel.materials.push("core:good/steel_section".into());
    (timber, steel)
}

fn workshop(sim: &Sim) -> &civ_agents::params::BuildingDef {
    let catalog = &sim.rules().catalog;
    &catalog.buildings[catalog
        .building_index("core:building/workshop")
        .expect("a workshop")]
}

fn sound(group: u32, now: civ_core::SimTime) -> civ_land::GroupCondition {
    civ_land::GroupCondition {
        group,
        quality: 1.0,
        loss: 0.0,
        installed: now,
        repaired: now,
        state: civ_land::GroupState::Sound,
    }
}

#[test]
fn a_skeleton_workshop_is_framed_in_sections_and_its_members_are_checked_as_steel() {
    use civ_agents::structure::{Loads, group_timber, margin};
    use civ_grammar::{GroupKind, Stage, frame_materials};
    let sim = world(2);
    let (_, home) = first_household(&sim);
    let catalog = &sim.rules().catalog;
    let def = workshop(&sim);
    let (timber, steel) = workshops(&sim, (home.0 + 20.0, home.1));
    let wood = civ_grammar::expand(&timber, &def.rules).expect("timber expands");
    let e = civ_grammar::expand(&steel, &def.rules).expect("the skeleton expands");
    // Its frame takes sections where the timber one took poles, at the program's density.
    let frame = &e.stages[Stage::Frame.index()].materials_kg;
    let sections = frame[usize::from(frame_materials::FRAME)];
    assert!(sections > 0.0);
    let poles =
        wood.stages[Stage::Frame.index()].materials_kg[usize::from(frame_materials::TIMBER)];
    assert!(
        (frame[usize::from(frame_materials::TIMBER)] + sections * 760.0 / 1119.0 - poles).abs()
            < 1e-6
    );
    // It used the skeleton's technique, and the timber one did not.
    let skeleton = technique(&sim, "skeleton_frame");
    assert!(civ_agents::rules::techniques_used(catalog, &steel, def).contains(&skeleton));
    assert!(
        !civ_agents::rules::techniques_used(catalog, &steel, def)
            .contains(&technique(&sim, "storeyed_frame"))
    );
    assert!(!civ_agents::rules::techniques_used(catalog, &timber, def).contains(&skeleton));
    // Its posts and joists are checked with steel's properties, its rafters with timber's.
    let section = catalog.goods[good(&sim, "steel_section")]
        .timber
        .expect("steel's");
    let oak = catalog.goods[good(&sim, "timber")]
        .timber
        .expect("timber's");
    let group = |e: &civ_grammar::Expansion, kind: GroupKind| {
        *e.groups.iter().find(|g| g.kind == kind).expect("a group")
    };
    for kind in [GroupKind::Posts, GroupKind::LoftJoists, GroupKind::TieBeams] {
        assert_eq!(
            group_timber(&catalog.goods, def, &group(&e, kind)),
            Some(section)
        );
        assert_eq!(
            group_timber(&catalog.goods, def, &group(&wood, kind)),
            Some(oak)
        );
    }
    assert_eq!(
        group_timber(&catalog.goods, def, &group(&e, GroupKind::RoofFrame)),
        Some(oak)
    );
    // Loaded until the timber member gives way, the steel one of the same size still stands.
    let now = sim.now();
    for (kind, load) in [
        (
            GroupKind::LoftJoists,
            Box::new(|pa: f64| Loads {
                loft_pa: pa,
                ..Loads::default()
            }) as Box<dyn Fn(f64) -> Loads>,
        ),
        (
            GroupKind::Posts,
            Box::new(|pa: f64| Loads {
                loft_pa: pa,
                ..Loads::default()
            }),
        ),
    ] {
        let check = |e: &civ_grammar::Expansion, pa: f64| {
            let g = group(e, kind);
            let t = group_timber(&catalog.goods, def, &g).expect("a material");
            margin(&g, &sound(g.id, now), e, &load(pa), &t).strength
        };
        let mut pa = 100.0;
        while check(&wood, pa) >= 1.0 {
            pa *= 1.05;
            assert!(pa < 1e9, "{kind:?} never gives way");
        }
        assert!(check(&wood, pa) > 0.9, "{kind:?}");
        assert!(
            check(&e, pa) > 1.2,
            "{kind:?}: {} at {pa} Pa",
            check(&e, pa)
        );
    }
}

#[test]
fn a_skeleton_workshop_is_raised_of_steel_sections() {
    use civ_grammar::{Stage, frame_materials};
    let mut sim = world(2);
    let (household, home) = first_household(&sim);
    let (_, steel) = workshops(&sim, (home.0 + 20.0, home.1));
    let def = workshop(&sim).clone();
    let e = civ_grammar::expand(&steel, &def.rules).expect("the skeleton expands");
    let sections = e.stages[Stage::Frame.index()].materials_kg[usize::from(frame_materials::FRAME)];
    // Framers, its ground worked and the frame to raise, with sections, poles and thatch at hand.
    let who = grown(&sim)
        .into_iter()
        .find(|&(h, _)| h == household)
        .map(|(_, p)| p)
        .expect("an adult");
    sim.introduce_technique(who, "core:technique/jointed_frame", false)
        .expect("introduced");
    let id = sim
        .place_building_for_tests(household, steel, 1)
        .expect("placed");
    set(
        &mut sim,
        household,
        &[
            ("steel_section", sections + 500.0),
            ("timber", 5_000.0),
            ("thatch", 5_000.0),
        ],
    );
    let section = good(&sim, "steel_section");
    let held = |sim: &Sim| {
        sim.people()
            .households
            .iter()
            .find(|(_, h)| h.id == household)
            .map(|(_, h)| h.stores[section])
            .expect("household")
    };
    let stage = |sim: &Sim| {
        sim.land()
            .buildings
            .iter()
            .find(|b| b.id == id)
            .expect("the workshop")
            .stage
    };
    for _ in 0..120 {
        if stage(&sim) > 1 {
            break;
        }
        sim.advance_minutes(24 * 60).expect("advances");
    }
    assert!(stage(&sim) > 1, "its frame was raised");
    // The frame took its sections.
    let used = sections + 500.0 - held(&sim);
    assert!(
        (used - sections).abs() < 1e-3 * sections,
        "{used} of {sections} kg"
    );
}

#[test]
fn builders_who_do_not_know_a_skeleton_design_as_before() {
    use civ_agents::build::{FireDesign, design_cautious, design_shape, shapes};
    let sim = world(2);
    let catalog = &sim.rules().catalog;
    for def in &catalog.buildings {
        let civ_grammar::ProgramRules::Frame(_) = &def.rules else {
            continue;
        };
        // Every shape a frame program is built in is a version 3 design of four materials, and
        // what each costs is what the content compiled.
        assert_eq!(shapes(def, &catalog.goods), def.shapes, "{}", def.id);
        for c in &def.shapes {
            let spec = design_shape(def, &catalog.goods, c.shape, (0.0, 0.0), None).expect("one");
            assert_eq!(
                spec.version,
                civ_grammar::FRAME_DESIGN_VERSION,
                "{}",
                def.id
            );
            assert_eq!(spec.materials.len(), civ_grammar::frame_materials::COUNT);
            assert_eq!(spec.params[civ_grammar::frame_params::POST_SYSTEM], 0);
            // Builders who know nothing of it, and those who know it and find it dearer than
            // timber (steel heavier at the same work), design the same.
            for knows in [false, true] {
                let fire = FireDesign {
                    knows_systems: [false, false, false, knows],
                    ..Default::default()
                };
                let designed = design_cautious(
                    def,
                    &catalog.goods,
                    c.shape,
                    (0.0, 0.0),
                    None,
                    1.0,
                    None,
                    Some(fire),
                );
                assert_eq!(designed.as_ref(), Some(&spec), "{} {knows}", def.id);
            }
        }
    }
    // A world lived where nobody knows it builds no version 4 frame.
    let mut sim = sim;
    for _ in 0..30 {
        sim.advance_minutes(24 * 60).expect("advances");
    }
    let skeleton = technique(&sim, "skeleton_frame");
    assert!(sim.people().people.iter().all(|(_, p)| !p.knows(skeleton)));
    for b in &sim.land().buildings {
        assert!(b.spec.version <= 3, "{:?}", b.spec);
    }
}

/// Workshop `spec`, finished, of the first household, burnt out in still dry air by the
/// observer's hand: its conditions as raised and as the fire left them, and its id.
fn burnt(
    sim: &mut Sim,
    spec: civ_grammar::BuildingSpec,
) -> (
    Vec<civ_land::GroupCondition>,
    Vec<civ_land::GroupCondition>,
    PermanentId,
) {
    let (household, _) = first_household(sim);
    let id = sim
        .place_building_for_tests(household, spec, civ_grammar::Stage::ALL.len() as u8)
        .expect("placed");
    let condition = |sim: &Sim| {
        sim.land()
            .buildings
            .iter()
            .find(|b| b.id == id)
            .expect("the workshop")
            .condition
            .clone()
    };
    let before = condition(sim);
    let w = &mut sim.land_mut_for_tests().weather;
    w.today.wind_ms = 0.0;
    w.today.gust_ms = 0.0;
    w.today.fuel_moisture = 0.12;
    sim.set_fire(id).expect("set alight");
    for _ in 0..120 {
        sim.advance_minutes(1).expect("advances");
        let out = sim
            .people()
            .fires
            .list
            .iter()
            .any(|f| f.building == id && f.ended.is_some());
        if out {
            break;
        }
    }
    let f = sim
        .people()
        .fires
        .list
        .iter()
        .find(|f| f.building == id)
        .expect("its fire");
    assert!(f.ended.is_some(), "it burnt out");
    (before, condition(sim), id)
}

#[test]
fn a_skeleton_softens_to_four_tenths_at_the_fires_heat_and_what_stands_regains_its_strength() {
    use civ_agents::structure::{Loads, group_timber, margin};
    use civ_grammar::GroupKind;
    let mut sim = world(2);
    let (_, home) = first_household(&sim);
    let (_, steel) = workshops(&sim, (home.0 + 40.0, home.1));
    let def = workshop(&sim).clone();
    let e = civ_grammar::expand(&steel, &def.rules).expect("the skeleton expands");
    let goods = sim.rules().catalog.goods.clone();
    // Steel sections soften to 0.4 of their strength (11-14 §2.1, at 600 °C); timber chars.
    let section = goods[good(&sim, "steel_section")].timber.expect("steel's");
    assert_eq!(section.burnt_keeps, Some(0.4));
    assert_eq!(
        goods[good(&sim, "timber")]
            .timber
            .expect("timber's")
            .burnt_keeps,
        None
    );
    let (before, after, id) = burnt(&mut sim, steel);
    let of = |e: &civ_grammar::Expansion, group: u32| {
        *e.groups.iter().find(|g| g.id == group).expect("a group")
    };
    let mut steel_groups = 0;
    for (was, now) in before.iter().zip(&after) {
        let g = of(&e, now.group);
        if group_timber(&goods, &def, &g) == Some(section) {
            // Its steel did not char; unloaded, it stood at the fire's heat and regained its
            // strength as it cooled (11-14 §1.6).
            steel_groups += 1;
            assert_eq!(now.loss, 0.0, "{:?}", g.kind);
            assert_ne!(now.state, civ_land::GroupState::Failed, "{:?}", g.kind);
            assert!(
                (f64::from(now.quality) - f64::from(was.quality)).abs() < 1e-6,
                "{:?}: {} of {}",
                g.kind,
                now.quality,
                was.quality
            );
        } else if g.kind == GroupKind::RoofFrame {
            // Its rafters are timber and char as before.
            assert!(now.loss > 0.0 && now.quality == was.quality);
        }
    }
    assert!(steel_groups >= 3, "{steel_groups} steel groups");
    // At the fire's heat, as the burn-out check weighs it, each member's margin is 0.4 of its
    // sound margin: under a loft load it carries with less than 2.5 to spare, it gives way.
    let load = Loads {
        loft_pa: 2_000.0,
        ..Loads::default()
    };
    for kind in [GroupKind::Posts, GroupKind::LoftJoists] {
        let g = *e.groups.iter().find(|g| g.kind == kind).expect("a group");
        let c = *before
            .iter()
            .find(|c| c.group == g.id)
            .expect("its condition");
        let hot = civ_land::GroupCondition {
            quality: c.quality * 0.4,
            ..c
        };
        let (sound, heated) = (
            margin(&g, &c, &e, &load, &section).strength,
            margin(&g, &hot, &e, &load, &section).strength,
        );
        assert!(sound > 1.0, "{kind:?}: {sound}");
        assert!(
            (heated / sound - 0.4).abs() < 1e-6,
            "{kind:?}: {heated} of {sound}"
        );
    }
    // The fire's page says its steel softened, and that its timber charred.
    let fire = sim
        .people()
        .fires
        .list
        .iter()
        .find(|f| f.building == id)
        .expect("its fire")
        .id;
    // Its words ride in the page's bytes as they are written.
    let bytes =
        civ_sim::frames::incidents::incident_response(&sim, civ_sim::frames::incidents::FIRE, fire)
            .expect("a page");
    let page = String::from_utf8_lossy(&bytes).into_owned();
    assert!(
        page.contains("Its steel softened in the fire to about 40% of its strength; what stood regained it as it cooled"),
        "{page}"
    );
    assert!(page.contains("Its timber charred"), "{page}");
}

#[test]
fn a_burnt_out_timber_workshop_chars_as_before() {
    let mut sim = world(2);
    let (_, home) = first_household(&sim);
    let (timber, _) = workshops(&sim, (home.0 + 40.0, home.1));
    let (before, after, _) = burnt(&mut sim, timber);
    // Exactly what the fire left before steel was given a share to keep (ADR-0022 §5): thirty
    // minutes' char on posts, joists and tie beams, more of the thinner rafters, half the infill,
    // no covering, and every quality as built.
    use civ_grammar::GroupKind;
    for (was, now) in before.iter().zip(&after) {
        assert_eq!(now.quality, was.quality, "{}", now.group);
        let loss = match GroupKind::of_group(now.group) {
            Some(GroupKind::Posts | GroupKind::LoftJoists | GroupKind::TieBeams) => 0.273_214_58,
            Some(GroupKind::RoofFrame) => 0.409_821_78,
            Some(GroupKind::Infill) => 0.5,
            Some(GroupKind::Covering) => 1.0,
            _ => 0.0,
        };
        assert!(
            (now.loss - loss).abs() < 1e-6,
            "{:?}: {} not {loss}",
            GroupKind::of_group(now.group),
            now.loss
        );
    }
}
