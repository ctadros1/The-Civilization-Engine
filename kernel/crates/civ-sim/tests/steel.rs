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
        charred: 0.0,
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
                    knows_systems: [false, false, false, knows, false],
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

/// The workshop of two bays with a loft its builders design, as a concrete frame (frame version
/// 5: post system 3, concrete and bar in the sixth and seventh slots).
fn cast_workshop(sim: &Sim, at: (f32, f32)) -> civ_grammar::BuildingSpec {
    let (timber, _) = workshops(sim, at);
    let mut cast = timber;
    cast.version = civ_agents::build::CONCRETE_VERSION;
    cast.params[civ_grammar::frame_params::POST_SYSTEM] = 3;
    cast.materials.extend([
        "core:good/steel_section".into(),
        "core:good/concrete".into(),
        "core:good/reinforcing_bar".into(),
    ]);
    cast
}

/// Loads a working building carries: thatch, a storm, goods in its lofts and on its floors, and
/// people on them (round figures within the core's).
fn working_loads() -> civ_agents::structure::Loads {
    civ_agents::structure::Loads {
        covering_pa: 400.0,
        peak_pa: 600.0,
        loft_pa: 1500.0,
        raised_pa: 1000.0,
        upper_pa: 1000.0,
        live_pa: 2000.0,
        infill_n: 0.0,
        casing_kg_m3: 0.0,
    }
}

#[test]
fn a_concrete_workshop_is_cast_of_concrete_and_bar_and_its_members_are_checked_as_concrete() {
    use civ_agents::structure::{group_timber, margin};
    use civ_grammar::{GroupKind, Stage, frame_materials};
    let sim = world(2);
    let (_, home) = first_household(&sim);
    let catalog = &sim.rules().catalog;
    let def = workshop(&sim);
    let (timber, _) = workshops(&sim, (home.0 + 20.0, home.1));
    let cast = cast_workshop(&sim, (home.0 + 20.0, home.1));
    let wood = civ_grammar::expand(&timber, &def.rules).expect("timber expands");
    let e = civ_grammar::expand(&cast, &def.rules).expect("the concrete frame expands");
    // Its frame takes concrete and bar at the program's figures, no sections.
    let frame = &e.stages[Stage::Frame.index()].materials_kg;
    let (concrete, bar) = (
        frame[usize::from(frame_materials::CONCRETE)],
        frame[usize::from(frame_materials::BAR)],
    );
    assert!(concrete > 0.0 && bar > 0.0);
    assert!((concrete / bar - 5760.0 / 307.0).abs() < 1e-9);
    assert_eq!(frame[usize::from(frame_materials::FRAME)], 0.0);
    // It used the concrete frame's technique, not the skeleton's.
    let used = civ_agents::rules::techniques_used(catalog, &cast, def);
    assert!(used.contains(&technique(&sim, "reinforced_concrete")));
    assert!(!used.contains(&technique(&sim, "skeleton_frame")));
    // Its posts, beams and joists are read as concrete, its rafters as timber.
    let concrete_t = catalog.goods[good(&sim, "concrete")]
        .timber
        .expect("concrete's");
    let oak = catalog.goods[good(&sim, "timber")]
        .timber
        .expect("timber's");
    let group = |e: &civ_grammar::Expansion, kind: GroupKind| {
        *e.groups.iter().find(|g| g.kind == kind).expect("a group")
    };
    for kind in [GroupKind::Posts, GroupKind::LoftJoists, GroupKind::TieBeams] {
        assert_eq!(
            group_timber(&catalog.goods, def, &group(&e, kind)),
            Some(concrete_t)
        );
    }
    assert_eq!(
        group_timber(&catalog.goods, def, &group(&e, GroupKind::RoofFrame)),
        Some(oak)
    );
    // Under a working building's loads every member stands, and its own weight tells: with nothing
    // in its loft, a cast loft joist carries more of itself than a timber one.
    let now = sim.now();
    let check = |e: &civ_grammar::Expansion, g: &civ_grammar::Group, loads| {
        let t = group_timber(&catalog.goods, def, g).expect("a material");
        margin(g, &sound(g.id, now), e, &loads, &t).strength
    };
    // Groups of a good the checks read: not thatch or wattle.
    let checked = |e: &civ_grammar::Expansion| -> Vec<civ_grammar::Group> {
        e.groups
            .iter()
            .filter(|g| group_timber(&catalog.goods, def, g).is_some())
            .copied()
            .collect()
    };
    for g in &checked(&e) {
        let m = check(&e, g, working_loads());
        assert!(m > 1.0, "{:?}: {m}", g.kind);
    }
    let bare = civ_agents::structure::Loads::default();
    let (cast_joist, wood_joist) = (
        group(&e, GroupKind::LoftJoists),
        group(&wood, GroupKind::LoftJoists),
    );
    let own = |e, g: &civ_grammar::Group, t: &civ_agents::structure::Timber| {
        // Strength over demand, read back to the demand under no load at a material's strength.
        t.bending_pa / check(e, g, bare)
    };
    assert!(
        own(&e, &cast_joist, &concrete_t) > 3.0 * own(&wood, &wood_joist, &oak),
        "a cast joist weighs 6,150 kg a cubic metre of its envelope against oak's 760"
    );
}

#[test]
fn a_six_storey_concrete_hall_its_builders_design_stands_its_own_checks() {
    use civ_agents::build::{FireDesign, Shape, design_cautious};
    use civ_agents::structure::{group_timber, margin};
    let sim = world(2);
    let (_, home) = first_household(&sim);
    let catalog = &sim.rules().catalog;
    // The hall as a program allowed to rise to twelve storeys, as the tall scene's longhouse is.
    let mut hall = catalog.buildings[catalog
        .building_index("core:building/hall")
        .expect("a hall")]
    .clone();
    if let civ_grammar::ProgramRules::Frame(r) = &mut hall.rules {
        r.storeys = (1, civ_grammar::MAX_STOREYS_V4);
    }
    let def = &hall;
    let civ_grammar::ProgramRules::Frame(r) = &def.rules else {
        panic!("a frame program")
    };
    let storeys = 6;
    let knows = FireDesign {
        knows_systems: [false, false, false, false, true],
        ..Default::default()
    };
    let spec = design_cautious(
        def,
        &catalog.goods,
        Shape::Bays {
            bays: r.bays.0.max(2),
            storeys,
            lofts: 0,
        },
        (home.0 + 30.0, home.1),
        None,
        1.0,
        None,
        Some(knows),
    )
    .expect("a design");
    assert_eq!(spec.version, civ_agents::build::CONCRETE_VERSION);
    assert_eq!(spec.params[civ_grammar::frame_params::POST_SYSTEM], 3);
    let e = civ_grammar::expand(&spec, &def.rules).expect("it expands");
    let now = sim.now();
    for g in e.groups.iter() {
        let Some(t) = group_timber(&catalog.goods, def, g) else {
            continue;
        };
        let m = margin(g, &sound(g.id, now), &e, &working_loads(), &t).strength;
        assert!(m > 1.0, "{:?} at {:#x}: {m}", g.kind, g.id);
    }
}

#[test]
fn a_concrete_frames_walls_wait_a_week_for_its_forms_to_be_stripped() {
    use civ_grammar::Stage;
    let mut sim = world(2);
    let (household, home) = first_household(&sim);
    let cast = cast_workshop(&sim, (home.0 + 20.0, home.1));
    let (timber, _) = workshops(&sim, (home.0 - 20.0, home.1));
    let walls = Stage::Walls.index() as u8;
    let cast_id = sim
        .place_building_for_tests(household, cast, walls)
        .expect("the concrete workshop placed");
    let timber_id = sim
        .place_building_for_tests(household, timber, walls)
        .expect("the timber workshop placed");
    let catalog = &sim.rules().catalog;
    let def = workshop(&sim);
    // The core's concrete is stripped at 7 days.
    let curve = catalog.goods[good(&sim, "concrete")]
        .timber
        .and_then(|t| t.cast)
        .expect("concrete cures");
    assert_eq!(curve.strip_days, 7.0);
    let building = |id| {
        sim.land()
            .buildings
            .iter()
            .find(|b| b.id == id)
            .expect("a building")
    };
    let b = building(cast_id);
    let until = civ_agents::build::curing_until(def, &catalog.goods, b).expect("it cures");
    assert_eq!(
        until.minutes() - b.stage_since.minutes(),
        7 * civ_core::time::MINUTES_PER_DAY
    );
    // A timber frame's walls wait for nothing, nor does a concrete frame at any other stage.
    assert_eq!(
        civ_agents::build::curing_until(def, &catalog.goods, building(timber_id)),
        None
    );
    let mut roofing = b.clone();
    roofing.stage = Stage::Roof.index() as u8;
    assert_eq!(
        civ_agents::build::curing_until(def, &catalog.goods, &roofing),
        None
    );
}

#[test]
fn a_concrete_workshops_rubble_infill_bears_itself_and_weighs_on_its_posts() {
    use civ_agents::structure::{Loads, group_timber, margin};
    use civ_grammar::{GroupKind, frame_params as fp};
    let sim = world(2);
    let (_, home) = first_household(&sim);
    let catalog = &sim.rules().catalog;
    let def = workshop(&sim);
    let civ_grammar::ProgramRules::Frame(r) = &def.rules else {
        panic!("a frame program")
    };
    let wattle = cast_workshop(&sim, (home.0 + 20.0, home.1));
    let mut rubble = wattle.clone();
    rubble.params[fp::WALL_KIND] = 1;
    rubble.params[fp::FOOTING] = 1;
    rubble.params[fp::WALL_CM] = r.mass_wall_cm.0;
    let framed = civ_grammar::expand(&wattle, &def.rules).expect("wattle infill");
    let e = civ_grammar::expand(&rubble, &def.rules).expect("rubble infill");
    let now = sim.now();
    // Each storey's infill bears only itself: its margin is its masonry's strength over its own
    // weight, far above one.
    let (strength, density) = civ_agents::rules::masonry(&def.rules, 1);
    let masonry = civ_agents::structure::Timber {
        bending_pa: 0.0,
        compression_pa: strength,
        stiffness_pa: 0.0,
        creep: 0.0,
        sustained: 1.0,
        density_kg_m3: density,
        burnt_keeps: None,
        cast: None,
        protection_min: 0.0,
    };
    let walls: Vec<_> = e
        .groups
        .iter()
        .filter(|g| g.kind == GroupKind::MassWall)
        .collect();
    assert!(!walls.is_empty());
    for g in &walls {
        let m = margin(g, &sound(g.id, now), &e, &working_loads(), &masonry).strength;
        assert!(m > 10.0, "infill bears itself: {m}");
    }
    // Its posts carry the infill's weight: weighed with it, their margin falls.
    let infill_n: f64 = walls
        .iter()
        .map(|g| {
            f64::from(g.section_cm[0])
                * f64::from(g.section_cm[1])
                * f64::from(g.length_cm)
                * 1e-6
                * density
                * civ_agents::structure::G
        })
        .sum();
    let posts = |e: &civ_grammar::Expansion, loads: Loads| {
        let g = e
            .groups
            .iter()
            .find(|g| g.kind == GroupKind::Posts)
            .expect("posts");
        let t = group_timber(&catalog.goods, def, g).expect("concrete");
        margin(g, &sound(g.id, now), e, &loads, &t).strength
    };
    let bare = posts(&e, working_loads());
    let loaded = posts(
        &e,
        Loads {
            infill_n,
            ..working_loads()
        },
    );
    assert!(loaded < bare, "{loaded} against {bare}");
    assert!(loaded > 1.0, "the posts still stand: {loaded}");
    assert_eq!(bare, posts(&framed, working_loads()));
}

#[test]
fn a_cased_skeleton_and_cast_concrete_keep_their_strength_through_a_burn_their_cover_outlasts() {
    use civ_grammar::frame_params as fp;
    let sim = world(2);
    let (_, home) = first_household(&sim);
    let catalog = &sim.rules().catalog;
    let def = workshop(&sim);
    let involved = sim.rules().people.fire.involved_minutes;
    assert_eq!(involved, 30.0, "a fully involved fire of the core's people");
    // A bare skeleton of version 5, and the same cased in concrete.
    let (_, steel) = workshops(&sim, (home.0 + 40.0, home.1));
    let mut bare = steel.clone();
    bare.version = 5;
    bare.materials.extend([
        "core:good/concrete".into(),
        "core:good/reinforcing_bar".into(),
    ]);
    let mut cased = bare.clone();
    cased.params[fp::ENCASEMENT] = 1;
    let soft = |spec: &civ_grammar::BuildingSpec, minutes: f64| {
        let e = civ_grammar::expand(spec, &def.rules).expect("expands");
        civ_agents::population::softening(&catalog.goods, def, spec, &e, minutes)
    };
    // Bare, its steel keeps 0.4 through any burn; cased, all of it through a burn up to its
    // casing's 90 minutes, and 0.4 through a longer one.
    let bare_soft = soft(&bare, involved);
    assert!(!bare_soft.is_empty());
    assert!(bare_soft.iter().all(|&(_, k)| k == 0.4));
    assert!(soft(&cased, involved).iter().all(|&(_, k)| k == 1.0));
    assert!(soft(&cased, 90.0).iter().all(|&(_, k)| k == 1.0));
    assert!(soft(&cased, 91.0).iter().all(|&(_, k)| k == 0.4));
    // The casing is the frame stage's concrete and hours.
    let (eb, ec) = (
        civ_grammar::expand(&bare, &def.rules).expect("bare"),
        civ_grammar::expand(&cased, &def.rules).expect("cased"),
    );
    let frame = |e: &civ_grammar::Expansion| e.stages[civ_grammar::Stage::Frame.index()].clone();
    let concrete = usize::from(civ_grammar::frame_materials::CONCRETE);
    assert_eq!(frame(&eb).materials_kg[concrete], 0.0);
    assert!(frame(&ec).materials_kg[concrete] > 0.0);
    assert!(frame(&ec).labour_h > frame(&eb).labour_h);
    // A concrete frame's cover holds an hour: through a 30-minute burn it keeps all its strength.
    let cast = cast_workshop(&sim, (home.0 + 20.0, home.1));
    assert!(soft(&cast, involved).iter().all(|&(_, k)| k == 1.0));
    assert!(soft(&cast, 61.0).iter().all(|&(_, k)| k == 0.75));
    // Casing is refused on a concrete frame or a skeleton of version 4.
    let mut cast_cased = cast.clone();
    cast_cased.params[fp::ENCASEMENT] = 1;
    assert!(civ_grammar::expand(&cast_cased, &def.rules).is_err());
    let mut old = steel;
    old.params[fp::ENCASEMENT] = 1;
    assert!(civ_grammar::expand(&old, &def.rules).is_err());
}

#[test]
fn a_protection_rule_binds_a_bare_skeleton_and_its_keepers_case_theirs() {
    use civ_agents::polity::RuleTemplate;
    use civ_grammar::frame_params as fp;
    let sim = world(2);
    let (_, home) = first_household(&sim);
    let catalog = &sim.rules().catalog;
    let def = workshop(&sim);
    let (timber, steel) = workshops(&sim, (home.0 + 40.0, home.1));
    // A bare skeleton is bound and breaks it; a timber frame is not bound.
    assert_eq!(
        civ_agents::rules::cased_for(&steel, &def.rules, 60.0),
        (true, false)
    );
    assert_eq!(
        civ_agents::rules::cased_for(&timber, &def.rules, 60.0),
        (false, false)
    );
    // Its keepers case it: written as version 5 with the program's concrete and bar, cased; it
    // keeps a rule of an hour, not one of longer than the program's 90 minutes of casing.
    let mut cased = steel.clone();
    assert!(civ_agents::build::case_skeleton(
        &mut cased,
        def,
        &catalog.goods,
        60.0
    ));
    assert_eq!(cased.version, civ_agents::build::CONCRETE_VERSION);
    assert_eq!(cased.params[fp::ENCASEMENT], 1);
    assert!(civ_grammar::expand(&cased, &def.rules).is_ok());
    assert_eq!(
        civ_agents::rules::cased_for(&cased, &def.rules, 60.0),
        (true, true)
    );
    assert_eq!(
        civ_agents::rules::cased_for(&cased, &def.rules, 120.0),
        (true, false)
    );
    let mut too_long = steel.clone();
    assert!(!civ_agents::build::case_skeleton(
        &mut too_long,
        def,
        &catalog.goods,
        120.0
    ));
    assert_eq!(too_long, steel, "left as it was");
    let mut wood = timber.clone();
    assert!(!civ_agents::build::case_skeleton(
        &mut wood,
        def,
        &catalog.goods,
        60.0
    ));
    // Its words.
    assert_eq!(
        RuleTemplate::ProtectedFrame { minutes: 60 }.words(),
        "a new steel frame must be cased against fire for at least 60 minutes"
    );
}

/// A member's envelope, cubic metres: its section's area (round for equal sides) times its
/// length, times its count.
fn envelope_m3(g: &civ_grammar::Group) -> f64 {
    let [w, d] = g.section_cm.map(|v| f64::from(v) / 100.0);
    let area = if w == d {
        std::f64::consts::PI * d * d / 4.0
    } else {
        w * d
    };
    area * f64::from(g.length_cm) / 100.0 * f64::from(g.count)
}

#[test]
fn a_cased_skeletons_posts_carry_its_casing_at_the_programs_figure() {
    use civ_agents::structure::{Loads, casing_kg_m3, group_timber, margin};
    use civ_grammar::{GroupKind, frame_materials, frame_params as fp};
    let sim = world(2);
    let (_, home) = first_household(&sim);
    let catalog = &sim.rules().catalog;
    let def = workshop(&sim);
    let civ_grammar::ProgramRules::Frame(r) = &def.rules else {
        panic!("a frame program")
    };
    // A bare skeleton of version 5, and the same cased in concrete.
    let (_, steel) = workshops(&sim, (home.0 + 40.0, home.1));
    let mut bare = steel;
    bare.version = 5;
    bare.materials.extend([
        "core:good/concrete".into(),
        "core:good/reinforcing_bar".into(),
    ]);
    let mut cased = bare.clone();
    cased.params[fp::ENCASEMENT] = 1;
    // The checks weigh its casing at the program's figure, the one its frame stage is reckoned
    // at; a bare skeleton and a timber frame have none.
    assert_eq!(casing_kg_m3(&cased, &def.rules), r.encasement_kg_per_m3);
    assert_eq!(r.encasement_kg_per_m3, 2250.0);
    assert_eq!(casing_kg_m3(&bare, &def.rules), 0.0);
    let (timber, _) = workshops(&sim, (home.0 + 40.0, home.1));
    assert_eq!(casing_kg_m3(&timber, &def.rules), 0.0);
    let e = civ_grammar::expand(&cased, &def.rules).expect("cased");
    let frame = |g: &&civ_grammar::Group| g.material == Some(frame_materials::FRAME);
    let stage_kg = e.stages[civ_grammar::Stage::Frame.index()].materials_kg
        [usize::from(frame_materials::CONCRETE)];
    let checked_kg: f64 =
        e.groups.iter().filter(frame).map(envelope_m3).sum::<f64>() * r.encasement_kg_per_m3;
    // The checks weigh it over the members as their groups draw them, as they weigh concrete;
    // the frame stage reckons more (the posts' feet in the ground among it).
    assert!(
        checked_kg > 0.5 * stage_kg && checked_kg <= stage_kg,
        "{checked_kg} against {stage_kg}"
    );
    // Its posts carry the casing of what stands on them (beams, plates and joists; a post's own
    // weight is never on it): exactly as the bare skeleton's would carry that weight laid on.
    let on_posts_n: f64 = e
        .groups
        .iter()
        .filter(frame)
        .filter(|g| g.kind != GroupKind::Posts)
        .map(envelope_m3)
        .sum::<f64>()
        * r.encasement_kg_per_m3
        * civ_agents::structure::G;
    assert!(on_posts_n > 0.0);
    let now = sim.now();
    let roof = Loads {
        covering_pa: 400.0,
        ..Loads::default()
    };
    let posts = |loads: Loads| {
        e.groups
            .iter()
            .filter(|g| g.kind == GroupKind::Posts)
            .map(|g| {
                let t = group_timber(&catalog.goods, def, g).expect("steel");
                margin(g, &sound(g.id, now), &e, &loads, &t).strength
            })
            .collect::<Vec<_>>()
    };
    let bare_m = posts(roof);
    let cased_m = posts(Loads {
        casing_kg_m3: casing_kg_m3(&cased, &def.rules),
        ..roof
    });
    let laid_on = posts(Loads {
        infill_n: on_posts_n,
        ..roof
    });
    for ((b, c), l) in bare_m.iter().zip(&cased_m).zip(&laid_on) {
        assert!(c < b, "{c} against {b}");
        assert!((c - l).abs() < 1e-9 * l, "{c} against {l}");
        assert!(*c > 1.0, "a cased skeleton still stands: {c}");
    }
}

/// The six-storey concrete hall its builders design (the hall as a program allowed to rise to
/// twelve storeys, as the tall scene's longhouse is), and its program.
fn six_storey_hall(sim: &Sim) -> (civ_agents::params::BuildingDef, civ_grammar::BuildingSpec) {
    use civ_agents::build::{FireDesign, Shape, design_cautious};
    let (_, home) = first_household(sim);
    let catalog = &sim.rules().catalog;
    let mut hall = catalog.buildings[catalog
        .building_index("core:building/hall")
        .expect("a hall")]
    .clone();
    if let civ_grammar::ProgramRules::Frame(r) = &mut hall.rules {
        r.storeys = (1, civ_grammar::MAX_STOREYS_V4);
    }
    let civ_grammar::ProgramRules::Frame(r) = &hall.rules else {
        panic!("a frame program")
    };
    let knows = FireDesign {
        knows_systems: [false, false, false, false, true],
        ..Default::default()
    };
    let spec = design_cautious(
        &hall,
        &catalog.goods,
        Shape::Bays {
            bays: r.bays.0.max(2),
            storeys: 6,
            lofts: 0,
        },
        (home.0 + 30.0, home.1),
        None,
        1.0,
        None,
        Some(knows),
    )
    .expect("a design");
    assert_eq!(spec.params[civ_grammar::frame_params::POST_SYSTEM], 3);
    (hall, spec)
}

#[test]
fn a_concrete_frame_shaken_or_in_a_gale_at_a_week_old_is_weaker_than_at_a_year() {
    use civ_agents::structure::{group_timber, lateral_margin, wind_margin};
    use civ_grammar::GroupKind;
    let sim = world(2);
    let (_, home) = first_household(&sim);
    let catalog = &sim.rules().catalog;
    let def = workshop(&sim);
    let curve = catalog.goods[good(&sim, "concrete")]
        .timber
        .and_then(|t| t.cast)
        .expect("concrete cures");
    let day = |d: i64| civ_core::SimTime::from_minutes(d * civ_core::time::MINUTES_PER_DAY);
    // Each post group shaken with 0.3 of the building's weight, its members put in place on day
    // 0 and shaken on day `age`, as the shaking reads them: a cast member at its age.
    let shaken = |spec: &civ_grammar::BuildingSpec, age: i64| -> Vec<f64> {
        let e = civ_grammar::expand(spec, &def.rules).expect("expands");
        e.groups
            .iter()
            .filter(|g| g.kind == GroupKind::Posts)
            .map(|g| {
                let t = group_timber(&catalog.goods, def, g)
                    .expect("a material")
                    .at_age(day(0), day(age));
                let c = sound(g.id, day(0));
                lateral_margin(g, &c, &e, &working_loads(), &t, 0.3, false).strength
            })
            .collect()
    };
    let cast = cast_workshop(&sim, (home.0 + 20.0, home.1));
    let (week, year) = (shaken(&cast, 7), shaken(&cast, 365));
    assert!(!week.is_empty());
    for (w, y) in week.iter().zip(&year) {
        assert!(w < y, "{w} against {y}");
        assert!((w / y - curve.share(7.0)).abs() < 1e-9, "{w} against {y}");
    }
    // A timber frame is as strong shaken at a week as at a year.
    let (timber, _) = workshops(&sim, (home.0 + 20.0, home.1));
    assert_eq!(shaken(&timber, 7), shaken(&timber, 365));
    // The storm's gust on the six-storey concrete hall's face, as the daily check reads its
    // posts: at a week, each storey's posts have less to spare than at a year.
    let (hall, spec) = six_storey_hall(&sim);
    let e = civ_grammar::expand(&spec, &hall.rules).expect("it expands");
    let q_pa = 600.0;
    let wind = |age: i64| -> Vec<f64> {
        e.groups
            .iter()
            .filter(|g| g.kind == GroupKind::Posts)
            .map(|g| {
                let t = group_timber(&catalog.goods, &hall, g)
                    .expect("concrete")
                    .at_age(day(0), day(age));
                wind_margin(g, &sound(g.id, day(0)), &e, &t, q_pa).strength
            })
            .collect()
    };
    let (week, year) = (wind(7), wind(365));
    assert!(
        week.iter().any(|m| m.is_finite()),
        "the gust bends its posts"
    );
    for (w, y) in week.iter().zip(&year) {
        if y.is_finite() {
            assert!(w < y, "{w} against {y}");
            assert!((w / y - curve.share(7.0)).abs() < 1e-9);
        }
    }
}

#[test]
fn a_concrete_workshops_rubble_infill_shakes_with_its_posts() {
    use civ_agents::structure::{Loads, group_timber, infill_weight_n, lateral_margin};
    use civ_grammar::{GroupKind, frame_params as fp};
    let sim = world(2);
    let (_, home) = first_household(&sim);
    let catalog = &sim.rules().catalog;
    let def = workshop(&sim);
    let civ_grammar::ProgramRules::Frame(r) = &def.rules else {
        panic!("a frame program")
    };
    let wattle = cast_workshop(&sim, (home.0 + 20.0, home.1));
    let mut rubble = wattle.clone();
    rubble.params[fp::WALL_KIND] = 1;
    rubble.params[fp::FOOTING] = 1;
    rubble.params[fp::WALL_CM] = r.mass_wall_cm.0;
    let framed = civ_grammar::expand(&wattle, &def.rules).expect("wattle infill");
    let e = civ_grammar::expand(&rubble, &def.rules).expect("rubble infill");
    // The daily check and the shaking weigh the same infill: none with wattle.
    let infill_n = infill_weight_n(&e, &def.rules);
    assert!(infill_n > 0.0);
    assert_eq!(infill_weight_n(&framed, &def.rules), 0.0);
    let now = sim.now();
    let posts = |e: &civ_grammar::Expansion, loads: Loads| -> Vec<f64> {
        e.groups
            .iter()
            .filter(|g| g.kind == GroupKind::Posts)
            .map(|g| {
                let t = group_timber(&catalog.goods, def, g).expect("concrete");
                lateral_margin(g, &sound(g.id, now), e, &loads, &t, 0.3, false).strength
            })
            .collect()
    };
    // Shaken with its infill's mass, its posts meet more shear and have less to spare than the
    // same frame shaken as if it had none, which stands as the wattle-filled one does.
    let without = posts(&e, working_loads());
    let with = posts(
        &e,
        Loads {
            infill_n,
            ..working_loads()
        },
    );
    assert_eq!(without, posts(&framed, working_loads()));
    for (w, o) in with.iter().zip(&without) {
        assert!(w < o, "{w} against {o}");
    }
}

/// A copy of `sim` through a save and a load.
fn copied(sim: &mut Sim) -> Sim {
    let d = tempfile::tempdir().expect("tempdir");
    let saves =
        commons_persist::SaveDir::create(d.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = civ_sim::persist::save(sim, &saves, commons_persist::SaveKind::Manual, "steel")
        .expect("save");
    civ_sim::persist::load(&saved.path, content()).expect("load")
}

#[test]
fn a_shaking_brings_down_a_concrete_workshop_a_week_old_that_it_would_not_at_a_year() {
    use civ_agents::quake::Outcome;
    let mut sim = world(2);
    let (household, home) = first_household(&sim);
    let cast = cast_workshop(&sim, (home.0 + 20.0, home.1));
    let id = sim
        .place_building_for_tests(household, cast, civ_grammar::Stage::ALL.len() as u8)
        .expect("the concrete workshop placed");
    let mut base = copied(&mut sim);
    // Each copy is shaken as its first event, so the workshop's draw is the same in every one;
    // its members were put in place `age` days before.
    let shaken = |base: &mut Sim, age: i64, strength: f64| {
        let mut copy = copied(base);
        let now = copy.now().minutes();
        let b = copy
            .land_mut_for_tests()
            .buildings
            .iter_mut()
            .find(|b| b.id == id)
            .expect("the workshop");
        for c in &mut b.condition {
            c.installed =
                civ_core::SimTime::from_minutes(now - age * civ_core::time::MINUTES_PER_DAY);
        }
        copy.shake(strength, None).expect("shake");
        copy.people()
            .quakes
            .list
            .last()
            .expect("a record")
            .shaken
            .iter()
            .find(|s| s.building == id)
            .expect("the workshop is reached")
            .outcome
    };
    // The least strength, in steps of a twentieth, that brings it down a week old; a year old,
    // its posts carry the 0.70 of their strength's shortfall and it stands.
    let mut strength = 0.1_f64;
    loop {
        assert!(strength <= 0.6, "no shaking brings the young workshop down");
        match shaken(&mut base, 7, strength) {
            Outcome::Fell => break,
            _ => strength *= 1.05,
        }
    }
    assert_eq!(
        shaken(&mut base, 365, strength),
        Outcome::Stood,
        "at {strength}"
    );
}
