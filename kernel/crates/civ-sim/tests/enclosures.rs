//! Enclosures (M6c slice BK, step one): a ring of palisade round a settlement's hearth is laid on
//! the walking grid. A standing sector is crossed only by climbing, slowly; an open gate is the
//! ground and a shut one is the palisade; a sector not yet built, or rotted through, bars nothing;
//! and the line is saved and loaded with the world.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::ChronicleKind;
use civ_agents::crime::{Belief, Incident, KnownAmount, Outcome, Source};
use civ_agents::polity::{LawStatus, PolicyKind};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_land::enclosures::{Enclosure, ring};
use civ_sim::{NewWorld, Sim, persist};
use civ_world::nav::RouteResult;

const PRESET: &str = "core:worldgen/river_valley";
const DAY: i64 = 24 * 60;

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

/// A village of world `seed`, with a fixed identity so it lives the same life every run.
fn village(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Enclosure".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

/// The time to walk between two cells on the walking grid as it stands, with nobody's trails.
fn route(sim: &Sim, from: u32, to: u32) -> Option<(Vec<u32>, f32)> {
    match sim.nav().route(
        &sim.map().elevation,
        from as usize,
        to as usize,
        &|_| 0.0,
        2_000_000,
    ) {
        RouteResult::Found(r) => {
            let secs = r.seconds.last().copied().unwrap_or(0.0);
            Some((r.cells, secs))
        }
        _ => None,
    }
}

fn cell_of(sim: &Sim, p: (f32, f32)) -> u32 {
    civ_agents::population::cell_of(sim.map(), p) as u32
}

/// A ring of `radius_m` round the first settlement's hearth with one gate, facing away from a
/// walkable cell outside it that the hearth reaches: the enclosure (not yet worked), the hearth's
/// cell and the cell outside.
fn site(sim: &mut Sim, radius_m: f32) -> (Enclosure, u32, u32) {
    let s = &sim.land().settlements[0];
    let (settlement, hearth) = (s.id, s.hearth_m);
    let cell_m = sim.map().cell_size_m;
    let home = cell_of(sim, hearth);
    let far = radius_m + 5.0 * cell_m;
    let (bearing, outside) = (0..16)
        .map(|k| k as f32 * std::f32::consts::TAU / 16.0 + std::f32::consts::PI)
        .find_map(|t| {
            let c = cell_of(sim, (hearth.0 + far * t.cos(), hearth.1 + far * t.sin()));
            (sim.nav().walkable(c as usize) && route(sim, home, c).is_some()).then_some((t, c))
        })
        .expect("somewhere outside the line the hearth reaches");
    let sectors = ring(
        hearth,
        radius_m,
        20.0,
        1,
        bearing + std::f32::consts::PI,
        2.0,
        cell_m,
        sim.map().width,
        sim.map().height,
    );
    let id = sim.allocate_id_for_tests();
    let enclosure = Enclosure {
        id,
        owner: settlement,
        law: settlement,
        settlement,
        centre: hearth,
        radius_m,
        sectors,
        work_h: 0.0,
        shares: Vec::new(),
        crew: (0, 0),
        begun: sim.now(),
        finished: None,
        upkeep_h: 0.0,
    };
    (enclosure, home, outside)
}

/// Puts `enclosure` in the world in place of any there, and lays the walking grid again.
fn lay(sim: &mut Sim, enclosure: &Enclosure) {
    let land = sim.land_mut_for_tests();
    land.enclosures.list = vec![enclosure.clone()];
    land.enclosures.changed();
    sim.lay_decks_for_tests();
}

/// The cells the enclosures bar now.
fn barred(sim: &Sim) -> Vec<u32> {
    sim.land()
        .enclosures
        .barriers(0.05)
        .into_iter()
        .map(|b| b.0)
        .collect()
}

#[test]
fn a_standing_line_is_climbed_slowly_an_open_gate_is_walked_through_and_a_shut_one_is_not() {
    let mut sim = village(3);
    sim.advance_minutes(DAY).expect("lives");
    let radius = 8.0 * sim.map().cell_size_m;
    let (mut enclosure, home, outside) = site(&mut sim, radius);
    let (_, free) = route(&sim, home, outside).expect("the way out");

    // Not yet worked: nothing stands, nothing is barred.
    lay(&mut sim, &enclosure);
    assert!(barred(&sim).is_empty());
    let (_, unbuilt) = route(&sim, home, outside).expect("the way out");
    assert!(
        (unbuilt - free).abs() < 1e-3,
        "{unbuilt} s against {free} s"
    );

    // Whole, its gate open on the far side: the way out goes round through the gate.
    enclosure.work_h = enclosure.labour_h();
    assert!(enclosure.whole());
    lay(&mut sim, &enclosure);
    assert!(!barred(&sim).is_empty());
    let (_, open) = route(&sim, home, outside).expect("out through the gate");
    assert!(open > free, "round by the gate {open} s, straight {free} s");

    // Its gate shut: every way out is a climb, slower than the gate was.
    for s in &mut enclosure.sectors {
        if let Some(g) = &mut s.gate {
            g.open = false;
        }
    }
    let before = sim.nav().travel_revision();
    lay(&mut sim, &enclosure);
    assert_ne!(
        sim.nav().travel_revision(),
        before,
        "routes planned before are stale"
    );
    let (cells, shut) = route(&sim, home, outside).expect("over the line");
    assert!(
        shut > open,
        "over the palisade {shut} s, through the gate {open} s"
    );
    let bars = barred(&sim);
    assert!(cells.iter().any(|c| bars.contains(c)), "it climbs the line");

    // Rotted through, every sector is a gap again.
    let mut gapped = enclosure.clone();
    for s in &mut gapped.sectors {
        s.loss = 1.0;
    }
    lay(&mut sim, &gapped);
    let (_, through) = route(&sim, home, outside).expect("through the gaps");
    assert!(
        (through - free).abs() < 1e-3,
        "{through} s against {free} s"
    );

    lay(&mut sim, &enclosure);
    assert!(sim.land().enclosures.problems().is_empty());

    // The observer is sent each sector, with its ends, state and gate.
    let bytes = civ_sim::frames::earthworks::earthworks_response(&sim);
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("wire");
    let sectors: Vec<_> = response
        .body_as_earthworks()
        .expect("earthworks")
        .works()
        .expect("works")
        .iter()
        .filter(|w| w.kind() == 5)
        .collect();
    assert_eq!(sectors.len(), enclosure.sectors.len());
    assert!(sectors.iter().all(|w| w.sector_state() == 1));
    let gate = sectors
        .iter()
        .find(|w| w.sector_gate() == 2)
        .expect("the shut gate");
    let i = gate.sector_index() as usize;
    assert_eq!(gate.sector_a().expect("a").x(), enclosure.sectors[i].a.0);
    assert!(gate.words().expect("words").contains("a gate, shut"));
    saves_and_goes_on_alike(&mut sim, DAY);
}

/// Two villages of world `seed`, with a fixed identity.
fn two_villages(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Neighbours".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: vec![30],
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

/// The households of settlement `s` with people, in id order.
fn households_of(sim: &Sim, s: PermanentId) -> Vec<PermanentId> {
    let mut out: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(s) && !h.members.is_empty())
        .map(|(_, h)| h.id)
        .collect();
    out.sort_unstable();
    out
}

/// Someone of household `taker_hh` (of settlement `from`) took `kcal` from household `held`
/// today, and the first of `held` knows who.
fn raided(sim: &mut Sim, taker_hh: PermanentId, from: PermanentId, held: PermanentId, kcal: f32) {
    let (now, day) = (sim.now(), sim.now().day_index());
    let taker = sim.people().household(taker_hh).expect("household").members[0];
    let holder = sim.people().household(held).expect("household").members[0];
    let id = {
        let order = &mut sim.people_mut_for_tests().order;
        let id = order.incidents.last().map_or(1, |i| i.id + 1);
        order.incidents.push(Incident {
            id,
            at: now,
            actor: taker,
            actor_household: taker_hh,
            target: held,
            place: (0.0, 0.0),
            outcome: Outcome::Taken,
            goods: Vec::new(),
            kcal,
            seen_by: Vec::new(),
            noticed: true,
            food_days: f32::NAN,
            richer: f32::NAN,
            across: Some(from),
        });
        order.amounts.push(KnownAmount {
            household: held,
            incident: id,
            kcal,
            lost: true,
            day,
        });
        id
    };
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.come_to_believe_for_tests(
            ctx,
            Belief {
                holder,
                incident: id,
                taker: Some(taker),
                source: Source::Told,
                from: None,
                origin: None,
                day,
            },
        );
    });
}

#[test]
fn a_village_raided_weighs_a_palisade_raises_it_by_law_and_its_stretches_stand_as_worked() {
    let mut sim = two_villages(3);
    sim.advance_minutes(DAY).expect("lives");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let (theirs, ours) = (households_of(&sim, a), households_of(&sim, b));
    // Nobody believes outsiders took anything: no palisade is worth weighing.
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.public_enclosure_for_tests(ctx, b))
            .is_none()
    );
    // Raided, and knowing by whom.
    let held = ours[0];
    raided(&mut sim, theirs[0], a, held, 400_000.0);
    let (site, keeps) = sim
        .with_ctx_for_tests(|pop, ctx| pop.public_enclosure_for_tests(ctx, b))
        .expect("a palisade worth weighing");
    let ep = sim.rules().people.enclosures.clone();
    assert!(site.radius_m as f64 >= ep.min_radius_m && site.inside >= 1);
    assert_eq!(usize::from(site.gates), ep.gates as usize);
    let hearth = sim.land().settlements[1].hearth_m;
    for &(h, kept, detour) in &keeps {
        let home = sim.people().household(h).expect("household").home;
        let inside = (home.0 - hearth.0).hypot(home.1 - hearth.1) < site.radius_m;
        if h == held && inside {
            assert!(kept > 0.0 && detour > 0.0, "{h:?} {kept} {detour}");
        } else {
            // Nobody else lost anything to outsiders; a home outside walks no detour.
            assert_eq!(kept, 0.0);
            assert_eq!(detour > 0.0, inside);
        }
    }
    // Those who believe they keep nothing pay their share without cover.
    let hearth_cell = civ_agents::population::cell_of(sim.map(), hearth) as u32;
    let gains = sim.with_ctx_for_tests(|pop, ctx| {
        pop.site_gains_for_tests(ctx, b, PolicyKind::Enclose, hearth_cell)
    });
    let of = |h: PermanentId| {
        gains
            .iter()
            .find(|g| g.0 == h)
            .map(|g| g.1)
            .expect("a gain")
    };
    for &h in ours.iter().filter(|&&h| h != held) {
        assert!(of(h) < 0.0, "{h:?} {}", of(h));
        assert!(of(held) > of(h));
    }
    // At the polity's review someone proposes it, the issue being what outsiders took.
    sim.people_mut_for_tests().every_adult_deliberates = true;
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == b)
        .expect("a polity");
    let mut proposed = None;
    let mut others = Vec::new();
    for _ in 0..12 {
        {
            let pop = sim.people_mut_for_tests();
            pop.polities[pi].gathering = None;
            for l in &mut pop.polities[pi].laws {
                if l.status == LawStatus::Proposed {
                    // Another answer to the same issue stands in force while this is weighed, so
                    // that it is not proposed again.
                    l.status = LawStatus::InForce;
                    others.push(l.id);
                }
            }
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.review_for_tests(ctx, b));
        match sim.people().polities[pi].agenda().cloned() {
            Some(l) if l.kind == PolicyKind::Enclose => {
                proposed = Some(l);
                break;
            }
            Some(_) => continue,
            None => break,
        }
    }
    {
        let pop = sim.people_mut_for_tests();
        for l in &mut pop.polities[pi].laws {
            if others.contains(&l.id) {
                l.status = LawStatus::Rejected;
            }
        }
    }
    let law = proposed.expect("a palisade proposed");
    assert_eq!(law.issue, civ_agents::polity::IssueKind::Raided);
    assert_eq!(
        sim.people().polities[pi].enclosure_site_of(law.id),
        Some(site)
    );
    let words = civ_sim::frames::word::law_text(&sim, &sim.people().polities[pi], &law);
    assert!(words.starts_with("a palisade "), "{words}");
    assert!(words.contains(&format!("{:.0} hours of work", site.labour_h)));
    // Passed: the polity lays out its line, nothing yet standing.
    sim.people_mut_for_tests().polities[pi]
        .law_mut(law.id)
        .expect("the law")
        .status = LawStatus::InForce;
    let known: Vec<PermanentId> = sim.people().people.iter().map(|(_, p)| p.id).collect();
    for p in known {
        sim.people_mut_for_tests().polities[pi]
            .law_mut(law.id)
            .expect("the law")
            .learn(p, 0);
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_public_enclosure_for_tests(ctx, pi, law.id));
    let id = sim.people().polities[pi]
        .enclosure_site_of(law.id)
        .and_then(|s| s.work)
        .expect("begun");
    let e = |sim: &Sim| {
        let land = sim.land();
        land.enclosures.list[land.enclosures.position(id).expect("its line")].clone()
    };
    assert_eq!((e(&sim).work_h, e(&sim).standing()), (0.0, 0));
    assert!((e(&sim).labour_h() - site.labour_h).abs() / site.labour_h < 0.05);
    assert!(sim.land().enclosures.problems().is_empty());
    // Someone of the raided household may work on it, at its first stretch, owing its share.
    let worker = sim.people().household(held).expect("household").members[0];
    let facts = sim
        .with_ctx_for_tests(|pop, ctx| pop.enclosure_facts_for_tests(ctx, worker))
        .expect("a palisade to work on");
    assert_eq!(facts.enclosure, id);
    assert!(
        facts.duty > 0.0 && facts.saves_per_hour > 0.0 && !facts.full,
        "{facts:?}"
    );
    // The work on its first stretch stands it, and changes walking.
    let first = e(&sim).sectors[0].labour_h;
    let before = sim.land().enclosures.revision();
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.enclosure_work_for_tests(ctx, worker, id, f64::from(first) * 0.5)
    });
    assert_eq!(e(&sim).standing(), 0, "half a stretch does not stand");
    assert_eq!(sim.land().enclosures.revision(), before);
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.enclosure_work_for_tests(ctx, worker, id, f64::from(first) * 0.5 + 0.01)
    });
    assert_eq!(e(&sim).standing(), 1);
    assert_ne!(sim.land().enclosures.revision(), before);
    assert!((e(&sim).worked_by(held) - first - 0.01).abs() < 1e-2);
    saves_and_goes_on_alike(&mut sim, 60);
    // Its people choose whether to work on it as they live (decide.rs weighs it): in a village
    // still building its homes and breaking its fields, nobody does in its first days.
    sim.advance_minutes(DAY).expect("lives");
    // Finished: its last stretch stands, and it is told.
    let left = f64::from(e(&sim).labour_h() - e(&sim).work_h) + 1.0;
    sim.with_ctx_for_tests(|pop, ctx| pop.enclosure_work_for_tests(ctx, worker, id, left));
    assert!(e(&sim).whole() && e(&sim).finished.is_some());
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|c| c.kind == ChronicleKind::Enclosure && c.name.contains("palisade"))
    );
    // Its gates stand open; the rest of its line bars walking.
    assert!(!sim.land().enclosures.barriers(0.05).is_empty());
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.enclosure_facts_for_tests(ctx, worker))
            .is_none(),
        "nothing left to work on"
    );
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.public_enclosure_for_tests(ctx, b))
            .is_none(),
        "a settlement with a line weighs no second"
    );
    // Raised, it is kept by the law that raised it: a stretch rotted past the mending share is
    // worked on again, the most rotted first, until its posts are new.
    let mend = sim.rules().people.enclosures.mend_loss as f32;
    let k = e(&sim)
        .sectors
        .iter()
        .position(|s| s.gate.is_none())
        .expect("a wall");
    {
        let land = sim.land_mut_for_tests();
        let i = land.enclosures.position(id).expect("its line");
        land.enclosures.list[i].sectors[k].loss = mend + 0.2;
    }
    let facts = sim
        .with_ctx_for_tests(|pop, ctx| pop.enclosure_facts_for_tests(ctx, worker))
        .expect("mending to do");
    let want = e(&sim).sectors[k].labour_h * (mend + 0.2);
    assert!((facts.left_h - f64::from(want)).abs() < 1e-2, "{facts:?}");
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.enclosure_work_for_tests(ctx, worker, id, f64::from(want) + 1.0)
    });
    assert_eq!(e(&sim).sectors[k].loss, 0.0);
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.enclosure_facts_for_tests(ctx, worker))
            .is_none()
    );
}

/// Advances `sim` to the next `minute` of the day.
fn until(sim: &mut Sim, minute: i64) {
    let now = sim.now().minute_of_day();
    let ahead = (minute - now).rem_euclid(DAY);
    sim.advance_minutes(if ahead == 0 { DAY } else { ahead })
        .expect("lives");
}

/// A whole palisade round settlement `s`'s hearth, owned by its polity under law `law`, put in
/// the world in place of any there.
fn raise_whole(sim: &mut Sim, s: PermanentId, law: PermanentId) -> PermanentId {
    let polity = sim
        .people()
        .polities
        .iter()
        .find(|p| p.settlement == s)
        .expect("a polity")
        .id;
    let hearth = sim
        .land()
        .settlements
        .iter()
        .find(|x| x.id == s)
        .expect("settlement")
        .hearth_m;
    let ep = sim.rules().people.enclosures.clone();
    let sectors = ring(
        hearth,
        80.0,
        ep.sector_m as f32,
        ep.gates as usize,
        0.0,
        ep.palisade_h_per_m as f32,
        sim.map().cell_size_m,
        sim.map().width,
        sim.map().height,
    );
    let id = sim.allocate_id_for_tests();
    let mut e = Enclosure {
        id,
        owner: polity,
        law,
        settlement: s,
        centre: hearth,
        radius_m: 80.0,
        sectors,
        work_h: 0.0,
        shares: Vec::new(),
        crew: (0, 0),
        begun: sim.now(),
        finished: Some(sim.now().day_index()),
        upkeep_h: 0.0,
    };
    e.work_h = e.labour_h();
    lay(sim, &e);
    id
}

#[test]
fn gates_shut_at_dusk_for_those_who_fear_outsiders_and_posts_rot_into_gaps_mended_by_work() {
    let mut sim = two_villages(3);
    sim.advance_minutes(DAY).expect("lives");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let (theirs, ours) = (households_of(&sim, a), households_of(&sim, b));
    let law = sim.allocate_id_for_tests();
    let id = raise_whole(&mut sim, b, law);
    let e = |sim: &Sim| {
        let land = sim.land();
        land.enclosures.list[land.enclosures.position(id).expect("its line")].clone()
    };
    let open = |sim: &Sim| {
        e(sim)
            .sectors
            .iter()
            .filter_map(|s| s.gate)
            .filter(|g| g.open)
            .count()
    };
    let gates = e(&sim).sectors.iter().filter(|s| s.gate.is_some()).count();
    assert!(gates >= 1 && open(&sim) == gates);
    // At dusk, nobody fearing outsiders: the gates stay open.
    until(&mut sim, 21 * 60);
    sim.with_ctx_for_tests(|pop, ctx| pop.keep_gates_for_tests(ctx));
    assert_eq!(open(&sim), gates);
    // Every household of the village raided by people of the other, and knowing by whom: at dusk
    // the household nearest each gate shuts it, and walking is laid again.
    for &h in &ours {
        raided(&mut sim, theirs[0], a, h, 5_000.0);
    }
    for &h in &ours {
        assert!(sim.with_ctx_for_tests(|pop, ctx| pop.fears_outsiders_for_tests(ctx, h)));
    }
    let before = sim.land().enclosures.revision();
    sim.with_ctx_for_tests(|pop, ctx| pop.keep_gates_for_tests(ctx));
    assert_eq!(open(&sim), 0, "all shut for the night");
    assert_ne!(sim.land().enclosures.revision(), before);
    let shut_bars = barred(&sim).len();
    // At first light they are opened again.
    until(&mut sim, 10 * 60);
    assert_eq!(open(&sim), gates, "the day's first light opened them");
    assert!(barred(&sim).len() < shut_bars);
    // Through the next dusk, kept by the scheduled keeping as the day turns.
    until(&mut sim, 22 * 60);
    assert_eq!(open(&sim), 0, "shut again at dusk");
    saves_and_goes_on_alike(&mut sim, 12 * 60);
    // Posts rot by the month, a stretch past the mending share asked mending, one rotted through
    // a gap that bars nothing.
    let mend = sim.rules().people.enclosures.mend_loss as f32;
    let k = e(&sim)
        .sectors
        .iter()
        .position(|s| s.gate.is_none())
        .expect("a wall");
    {
        let land = sim.land_mut_for_tests();
        let i = land.enclosures.position(id).expect("its line");
        for s in &mut land.enclosures.list[i].sectors {
            s.loss = mend - 0.001;
        }
        land.enclosures.list[i].sectors[k].loss = 0.999;
    }
    let before = sim.land().enclosures.revision();
    sim.with_ctx_for_tests(|pop, ctx| pop.wear_enclosures_for_tests(ctx));
    let now = e(&sim);
    assert!(now.sectors[k].gap(), "rotted through");
    assert!(now.sectors.iter().all(|s| s.loss >= mend));
    assert!(now.upkeep_h > 0.0 && now.mend_h(mend) > 0.0);
    assert_ne!(sim.land().enclosures.revision(), before);
    let bars_k = now.sectors[k].cells.clone();
    assert!(barred(&sim).iter().all(|c| !bars_k.contains(c) || {
        // A cell another standing stretch shares still bars.
        now.sectors
            .iter()
            .enumerate()
            .any(|(j, s)| j != k && now.bars(j) && s.cells.contains(c))
    }));
    // Mending the gap first closes it.
    let worker = sim.people().household(ours[0]).expect("household").members[0];
    let before = sim.land().enclosures.revision();
    let need = f64::from(now.sectors[k].labour_h);
    sim.with_ctx_for_tests(|pop, ctx| pop.enclosure_work_for_tests(ctx, worker, id, need));
    assert!(!e(&sim).sectors[k].gap());
    assert_ne!(sim.land().enclosures.revision(), before);
    assert!(e(&sim).worked_by(ours[0]) > 0.0);
    assert!(sim.land().enclosures.problems().is_empty());
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "ring").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(sim.land().enclosures, loaded.land().enclosures);
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(sim, &loaded);
    assert_eq!(sim.nav().travel_revision(), loaded.nav().travel_revision());
    sim.advance_minutes(minutes).expect("advances");
    loaded.advance_minutes(minutes).expect("advances");
    same(sim, &loaded);
}

/// A band of `n` grown people of settlement `a` called against household `target` (of `b`, its
/// home inside palisade `line`), coming up to the line on the side toward `a`'s hearth and
/// mustering at seven the next morning; and every household of `b` hears it was seen coming, so
/// that its gates are kept shut.
fn a_band_against_the_line(
    sim: &mut Sim,
    a: PermanentId,
    b: PermanentId,
    target: PermanentId,
    line: PermanentId,
    n: usize,
) -> u32 {
    use civ_agents::war::{Band, BandPurpose, BandStage};
    use civ_agents::word::{Claim, ClaimKind};
    let now = sim.now();
    let adult = sim.rules().people.bands.min_age;
    let hearth_a = sim
        .land()
        .settlements
        .iter()
        .find(|s| s.id == a)
        .expect("settlement")
        .hearth_m;
    let e = {
        let land = sim.land();
        land.enclosures.list[land.enclosures.position(line).expect("the line")].clone()
    };
    let (dx, dy) = (hearth_a.0 - e.centre.0, hearth_a.1 - e.centre.1);
    let len = dx.hypot(dy);
    let r = e.radius_m + 10.0;
    let approach = (e.centre.0 + dx / len * r, e.centre.1 + dy / len * r);
    let pop = sim.people_mut_for_tests();
    let place = pop.household(target).expect("household").home;
    let mut adults: Vec<PermanentId> = pop
        .people
        .iter()
        .filter(|(_, p)| p.age_years(now) >= adult && p.age_years(now) <= 50.0)
        .filter(|(_, p)| pop.household(p.household).and_then(|x| x.settlement) == Some(a))
        .map(|(_, p)| p.id)
        .collect();
    adults.sort_unstable();
    adults.truncate(n);
    let id = pop.order.bands.last().map_or(1, |x| x.id + 1);
    pop.order.bands.push(Band {
        id,
        organizer: adults[0],
        settlement: a,
        purpose: BandPurpose::Take,
        target,
        place,
        called: now.minutes(),
        muster: (now.day_index() + 1) * DAY + 7 * 60,
        members: adults,
        asked: Vec::new(),
        mustered: Vec::new(),
        arrived: Vec::new(),
        home: Vec::new(),
        stage: BandStage::Called,
        since: now.minutes(),
        clash: None,
        incident: None,
        law: None,
        taken: Vec::new(),
        taken_kcal: 0.0,
        approach: Some(approach),
    });
    let day = now.day_index();
    let claim = pop.word.make(Claim {
        id: 0,
        kind: ClaimKind::BandSeen,
        settlement: a,
        day,
        subject: None,
        grievance: None,
        suspected: None,
        fire: None,
        event: Some(id),
    });
    let hearers: Vec<PermanentId> = pop
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(b))
        .flat_map(|(_, x)| x.members.clone())
        .collect();
    for p in hearers {
        pop.word.hear(p, claim, day, None, Some(p));
    }
    id
}

/// A household of settlement `s` whose home lies inside palisade `line`.
fn inside(sim: &Sim, s: PermanentId, line: PermanentId) -> PermanentId {
    let land = sim.land();
    let e = &land.enclosures.list[land.enclosures.position(line).expect("the line")];
    households_of(sim, s)
        .into_iter()
        .find(|&h| e.inside(sim.people().household(h).expect("household").home))
        .expect("a home inside the line")
}

/// Advances `sim` until band `id`'s stage is `stage`, at most `minutes`; whether it got there.
fn until_stage(sim: &mut Sim, id: u32, stage: civ_agents::war::BandStage, minutes: i64) -> bool {
    let mut left = minutes;
    while left > 0 {
        if sim
            .people()
            .order
            .bands
            .iter()
            .any(|b| b.id == id && b.stage == stage)
        {
            return true;
        }
        sim.advance_minutes(10).expect("lives");
        left -= 10;
    }
    false
}

fn band(sim: &Sim, id: u32) -> civ_agents::war::Band {
    sim.people()
        .order
        .bands
        .iter()
        .find(|b| b.id == id)
        .expect("the band")
        .clone()
}

#[test]
fn a_band_that_comes_up_to_a_shut_line_camps_outside_it_and_goes_in_by_a_gap() {
    use civ_agents::war::BandStage;
    let mut sim = two_villages(3);
    sim.advance_minutes(DAY).expect("lives");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let law = sim.allocate_id_for_tests();
    let line = raise_whole(&mut sim, b, law);
    let target = inside(&sim, b, line);
    let id = a_band_against_the_line(&mut sim, a, b, target, line, 4);
    // Seen coming, the gates are shut by those who fear it; it comes up to the line and camps
    // outside it, nobody struck.
    assert!(until_stage(&mut sim, id, BandStage::Camped, 2 * DAY));
    assert!(
        sim.land().enclosures.list[0]
            .sectors
            .iter()
            .filter_map(|s| s.gate)
            .all(|g| !g.open),
        "every gate shut"
    );
    let camp = band(&sim, id);
    assert!(camp.clash.is_none());
    let approach = camp.approach.expect("an approach");
    for &m in camp.arrived.iter().filter(|m| !camp.home.contains(m)) {
        let p = sim.people().person(m).expect("alive").pos;
        assert!(
            (p.0 - approach.0).hypot(p.1 - approach.1) < 30.0,
            "{m:?} at {p:?}, the camp at {approach:?}"
        );
    }
    saves_and_goes_on_alike(&mut sim, 60);
    // A stretch of the line rots through: at the next look, they go in and meet whoever stands.
    {
        let land = sim.land_mut_for_tests();
        let i = land.enclosures.position(line).expect("the line");
        let k = land.enclosures.list[i]
            .sectors
            .iter()
            .position(|s| s.gate.is_none())
            .expect("a wall");
        land.enclosures.list[i].sectors[k].loss = 1.0;
        land.enclosures.changed();
    }
    sim.lay_decks_for_tests();
    let camp_minutes = i64::from(sim.rules().people.bands.camp_minutes);
    let mut met = false;
    for _ in 0..(camp_minutes / 10 + 2) {
        sim.advance_minutes(10).expect("lives");
        if band(&sim, id).clash.is_some() {
            met = true;
            break;
        }
    }
    assert!(met, "through the gap: {:?}", band(&sim, id).stage);
}

#[test]
fn a_band_camped_by_a_line_that_stays_shut_goes_home_one_by_one() {
    use civ_agents::war::BandStage;
    let mut sim = two_villages(3);
    sim.advance_minutes(DAY).expect("lives");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let law = sim.allocate_id_for_tests();
    let line = raise_whole(&mut sim, b, law);
    let target = inside(&sim, b, line);
    let id = a_band_against_the_line(&mut sim, a, b, target, line, 4);
    assert!(until_stage(&mut sim, id, BandStage::Camped, 2 * DAY));
    let came = band(&sim, id).arrived.len();
    assert!(came >= 2);
    // Each chooses when to go; the line never opens: they go home over the following hours and
    // days, not all at once, and nobody is struck.
    let mut seen = Vec::new();
    for _ in 0..(4 * DAY / 30) {
        sim.advance_minutes(30).expect("lives");
        let x = band(&sim, id);
        let left = x.arrived.iter().filter(|m| x.home.contains(m)).count();
        if seen.last() != Some(&left) {
            seen.push(left);
        }
        if x.stage.over() {
            break;
        }
    }
    let x = band(&sim, id);
    assert!(x.stage.over(), "{:?} after four days", x.stage);
    assert!(x.clash.is_none(), "the line kept them out");
    assert!(
        seen.len() >= 3,
        "they went home in more than one step: {seen:?}"
    );
}
