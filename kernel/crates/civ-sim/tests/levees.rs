//! Flood banks people choose to build together (M6b slice BG; ADR-0022 §11).
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim, persist};
use std::{path::Path, sync::OnceLock};

const DAY: i64 = 1440;

fn content() -> &'static ContentRegistry {
    static C: OnceLock<ContentRegistry> = OnceLock::new();
    C.get_or_init(|| {
        civ_content::load(
            &civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("root"),
        )
        .registry
        .expect("content")
    })
}

fn village(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Bankside".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
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

fn households(sim: &Sim) -> Vec<PermanentId> {
    let mut out: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .map(|(_, h)| h.id)
        .collect();
    out.sort_unstable();
    out
}

/// The reach nearest settlement `s`'s hearth.
fn reach_near(sim: &Sim, at: (f32, f32)) -> usize {
    let map = sim.map();
    let cell = f64::from(map.cell_size_m);
    map.reaches
        .iter()
        .enumerate()
        .min_by(|a, b| {
            let d = |r: &civ_world::RiverReach| {
                r.cells
                    .iter()
                    .map(|&c| {
                        let x = (f64::from(c % map.width) + 0.5) * cell;
                        let y = (f64::from(c / map.width) + 0.5) * cell;
                        (x - f64::from(at.0)).hypot(y - f64::from(at.1))
                    })
                    .fold(f64::INFINITY, f64::min)
            };
            d(a.1).total_cmp(&d(b.1))
        })
        .map(|r| r.0)
        .expect("a reach")
}

fn land_problems(sim: &Sim) -> Vec<String> {
    sim.land()
        .problems(sim.map(), sim.rules().land.habitats.len(), u64::MAX)
}

fn land_ok(sim: &Sim) -> bool {
    land_problems(sim).is_empty()
}

fn same(a: &Sim, b: &Sim) {
    let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
}

fn reloaded(sim: &mut Sim) -> Sim {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "bank").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    same(sim, &loaded);
    loaded
}

/// A lived village of seed 3, a high water raised on the reach beside it once its homes stand,
/// lived through until it has gone down.
fn flooded_village() -> (Sim, PermanentId, usize) {
    let mut sim = village(3);
    sim.advance_minutes(40 * DAY).expect("advances");
    let all = households(&sim);
    let settlement = sim
        .people()
        .household(all[0])
        .and_then(|x| x.settlement)
        .expect("settled");
    let hearth = sim
        .land()
        .settlements
        .iter()
        .find(|s| s.id == settlement)
        .expect("the settlement")
        .hearth_m;
    let reach = reach_near(&sim, hearth);
    // A hundred and fifty cubic metres a second over what the reach holds within its banks:
    // water deep enough at some homes that a bank is worth its work only if it comes often.
    let bankfull = sim.land().floodplain.reaches[reach].bankfull_discharge_m3s as f32;
    let natural = sim.land().reach_discharges[reach].max(bankfull);
    sim.raise_river(reach, natural + 150.0, 6).expect("raise");
    sim.advance_minutes(7 * DAY).expect("advances");
    (sim, settlement, reach)
}

#[test]
fn a_flood_bank_the_gathering_asks_for_is_laid_out_from_what_floods_cost_and_built_in_shares() {
    use civ_agents::polity::{IssueKind, LawStatus, PolicyKind};
    let (mut sim, settlement, reach) = flooded_village();
    let all = households(&sim);
    // Households saw what the water cost them: their own stores spoiled, at their own places.
    let lost: Vec<PermanentId> = all
        .iter()
        .copied()
        .filter(|&h| {
            !sim.people()
                .household(h)
                .expect("a household")
                .flood_losses
                .is_empty()
        })
        .collect();
    assert!(!lost.is_empty() && lost.len() < all.len(), "{lost:?}");
    for &h in &lost {
        let x = sim.people().household(h).expect("a household");
        assert!(
            x.flood_losses
                .iter()
                .all(|l| l.depth_cm > 0 && l.kcal >= 0.0)
        );
    }
    // One flood, remembered over the years people remember, does not repay a bank's work.
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.public_bank_for_tests(ctx, settlement))
            .is_none()
    );
    // As though it had flooded so in each of the last five years.
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        let once = x.flood_losses.clone();
        let mut all_years = Vec::new();
        for k in (1..5).rev() {
            all_years.extend(once.iter().map(|l| civ_agents::FloodLoss {
                day: l.day - 365 * k,
                ..*l
            }));
        }
        all_years.extend(once);
        x.flood_losses = all_years;
    }
    let (site, saves) = sim
        .with_ctx_for_tests(|pop, ctx| pop.public_bank_for_tests(ctx, settlement))
        .expect("a bank worth building together");
    // It keeps out the water of a reach the raised flow ran through.
    assert!(sim.map().reaches.iter().any(|r| r.id == site.reach));
    let _ = reach;
    // Only households that lost something it would keep dry gain by it.
    assert!(!saves.is_empty() && saves.iter().all(|s| lost.contains(&s.0) && s.1 > 0.0));
    let life = sim.rules().people.levees.life_years;
    assert!(saves.iter().map(|s| s.1).sum::<f64>() * life > site.labour_h);
    // Its height stands above the deepest water its people remember at the places it keeps.
    assert!(site.height_m >= sim.rules().people.levees.min_height_m);
    let cell = {
        let mid = (
            ((site.a[0] + site.b[0]) / 2.0) as f32,
            ((site.a[1] + site.b[1]) / 2.0) as f32,
        );
        let map = sim.map();
        (mid.1 / map.cell_size_m) as u32 * map.width + (mid.0 / map.cell_size_m) as u32
    };
    let gains = sim.with_ctx_for_tests(|pop, ctx| {
        pop.site_gains_for_tests(ctx, settlement, PolicyKind::BuildBank, cell)
    });
    for (h, g) in &gains {
        if saves.iter().any(|s| s.0 == *h) {
            assert!(*g > 0.0, "{h:?} {g}");
        } else {
            assert!(*g < 0.0, "{h:?} {g}: asked its share, kept nothing");
        }
    }
    // At the polity's review someone proposes it, the issue being what floods cost.
    sim.people_mut_for_tests().every_adult_deliberates = true;
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == settlement)
        .expect("a polity");
    let mut proposed = None;
    for _ in 0..12 {
        {
            let pop = sim.people_mut_for_tests();
            pop.polities[pi].gathering = None;
            for l in &mut pop.polities[pi].laws {
                if l.status == LawStatus::Proposed {
                    l.status = LawStatus::Rejected;
                }
            }
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.review_for_tests(ctx, settlement));
        match sim.people().polities[pi].agenda().cloned() {
            Some(l) if l.kind == PolicyKind::BuildBank => {
                proposed = Some(l);
                break;
            }
            Some(_) => continue,
            None => break,
        }
    }
    let law = proposed.expect("a bank proposed");
    assert_eq!(law.issue, IssueKind::Flooded);
    assert_eq!(sim.people().polities[pi].bank_site_of(law.id), Some(site));
    let what = civ_sim::frames::word::law_text(&sim, &sim.people().polities[pi], &law);
    assert_eq!(
        what,
        format!(
            "a bank {:.0} m long and {:.1} m high against floodwater, each household doing its \
             share of {:.0} hours of work",
            site.length_m(),
            site.height_m,
            site.labour_h
        )
    );
    // Passed: the polity lays it out, with no earth in it yet.
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
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_public_bank_for_tests(ctx, pi, law.id));
    let bank = sim.people().polities[pi]
        .bank_site_of(law.id)
        .and_then(|s| s.work)
        .expect("begun");
    let seg = *sim
        .land()
        .banks
        .iter()
        .find(|b| b.work == bank)
        .expect("its segment");
    let work = |sim: &Sim| {
        *sim.land()
            .earthworks
            .iter()
            .find(|e| e.id == bank)
            .expect("its earthwork")
    };
    let pit = |sim: &Sim| {
        *sim.land()
            .earthworks
            .iter()
            .find(|e| e.id == seg.borrow)
            .expect("its pit")
    };
    assert_eq!((work(&sim).done, pit(&sim).cut_m3), (0.0, 0.0));
    assert!(land_ok(&sim), "{:?}", land_problems(&sim));
    // Someone of a household that kept nothing works on it: the hours their earth took are
    // their household's given, and the earth placed was dug from the pit.
    let other = *all
        .iter()
        .find(|h| !lost.contains(h))
        .expect("one that lost nothing");
    let worker = sim.people().household(other).expect("a household").members[0];
    sim.with_ctx_for_tests(|pop, ctx| pop.levee_work_by_for_tests(ctx, worker, bank, 6.0));
    let given = sim.people().polities[pi].bank_given_by(law.id, other);
    assert!((given - 6.0).abs() < 1e-3, "{given}");
    let (w, p) = (work(&sim), pit(&sim));
    assert!(w.done > 0.0 && w.done < 1.0);
    assert!(
        (f64::from(w.cut_m3) - f64::from(p.cut_m3) * seg.compacted_per_bank).abs() < 1e-3,
        "{} {}",
        w.cut_m3,
        p.cut_m3
    );
    assert!(land_ok(&sim), "{:?}", land_problems(&sim));
    let mut sim = reloaded(&mut sim);
    // Its people may work on it as they choose: one of a household that has not given its
    // share weighs the gathering's word; once it has, only what the bank keeps for it. (Their
    // views of the ground from the hearth are found as they live.)
    sim.advance_minutes(60).expect("advances");
    let owing = sim
        .people()
        .household(lost[0])
        .expect("a household")
        .members[0];
    let facts = sim
        .with_ctx_for_tests(|pop, ctx| pop.levee_facts_for_tests(ctx, owing))
        .expect("a bank to work on");
    assert_eq!(facts.bank, bank);
    assert!(facts.duty > 0.0 && !facts.full, "{facts:?}");
    let given = sim
        .with_ctx_for_tests(|pop, ctx| pop.levee_facts_for_tests(ctx, worker))
        .expect("a bank to work on");
    assert!(
        given.duty > 0.0,
        "6 hours is less than its share: {given:?}"
    );
    let share = site.labour_h / all.len() as f64;
    sim.with_ctx_for_tests(|pop, ctx| pop.levee_work_by_for_tests(ctx, worker, bank, share));
    let given = sim
        .with_ctx_for_tests(|pop, ctx| pop.levee_facts_for_tests(ctx, worker))
        .expect("a bank to work on");
    assert_eq!(given.duty, 0.0, "its share given: {given:?}");
    // Finished: the last of its earth is in, and it is told.
    let left = site.labour_h * 2.0;
    sim.with_ctx_for_tests(|pop, ctx| pop.levee_work_by_for_tests(ctx, worker, bank, left));
    assert_eq!(work(&sim).done, 1.0);
    let name = sim
        .land()
        .settlements
        .iter()
        .find(|s| s.id == settlement)
        .expect("the settlement")
        .name
        .clone();
    let told = sim.people().chronicle.iter().any(|e| {
        e.name.starts_with(&format!(
            "{name}'s people finished the bank against floodwater"
        ))
    });
    assert!(told);
    assert!(land_ok(&sim), "{:?}", land_problems(&sim));
    let mut sim = reloaded(&mut sim);
    // Seepage damage left to stand: the bank wants mending, and nobody mends it unless a law
    // asks. Those it keeps water from gain by keeping it; the others are asked their share.
    let damage = |sim: &mut Sim, piping: f64| {
        let seg = sim
            .land_mut_for_tests()
            .banks
            .iter_mut()
            .find(|b| b.work == bank)
            .expect("its segment");
        seg.piping = piping;
    };
    damage(&mut sim, 0.4);
    // (The losses its households remember would also have them weigh a second, higher bank
    // beside it; forgotten here, so the keeping is what they weigh. What each reckoned this bank
    // saves it stays with the law that built it.)
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        x.flood_losses.clear();
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.look_over_banks_for_tests(ctx, pi));
    assert!(
        !sim.people().polities[pi].banks[0].1.mending,
        "no law keeps it yet"
    );
    let keep_gains = sim.with_ctx_for_tests(|pop, ctx| {
        pop.site_gains_for_tests(ctx, settlement, PolicyKind::KeepBank, cell)
    });
    assert!(!keep_gains.is_empty());
    for (h, g) in &keep_gains {
        if saves.iter().any(|s| s.0 == *h) {
            assert!(*g > 0.0, "{h:?} {g}");
        } else {
            assert!(
                *g < 0.0,
                "{h:?} {g}: asked its share of the mending, kept nothing"
            );
        }
    }
    let mut keeping = None;
    for _ in 0..12 {
        {
            let pop = sim.people_mut_for_tests();
            pop.polities[pi].gathering = None;
            for l in &mut pop.polities[pi].laws {
                if l.status == LawStatus::Proposed {
                    l.status = LawStatus::Rejected;
                }
            }
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.review_for_tests(ctx, settlement));
        match sim.people().polities[pi].agenda().cloned() {
            Some(l) if l.kind == PolicyKind::KeepBank => {
                keeping = Some(l);
                break;
            }
            Some(_) => continue,
            None => break,
        }
    }
    let keep = keeping.expect("a keeping proposed");
    assert_eq!(keep.issue, IssueKind::BankWorn);
    assert_eq!(
        sim.people().polities[pi].kept_bank_of(keep.id).map(|k| k.0),
        Some(law.id)
    );
    let what = civ_sim::frames::word::law_text(&sim, &sim.people().polities[pi], &keep);
    assert!(what.starts_with("the keeping of the bank"), "{what}");
    // Passed: it is looked over at once and its mending asked in equal shares.
    {
        let pop = sim.people_mut_for_tests();
        let l = pop.polities[pi].law_mut(keep.id).expect("the law");
        l.status = LawStatus::InForce;
    }
    let known: Vec<PermanentId> = sim.people().people.iter().map(|(_, p)| p.id).collect();
    for p in known {
        sim.people_mut_for_tests().polities[pi]
            .law_mut(keep.id)
            .expect("the law")
            .learn(p, 0);
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.look_over_banks_for_tests(ctx, pi));
    let kept = sim.people().polities[pi]
        .bank_site_of(law.id)
        .expect("site");
    assert!(kept.mending && kept.upkeep_h > 0.0, "{kept:?}");
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.name.contains("bank they keep against floodwater seeping"))
    );
    // Looked over again while it is being mended, nothing more is asked.
    let asked = kept.upkeep_h;
    sim.with_ctx_for_tests(|pop, ctx| pop.look_over_banks_for_tests(ctx, pi));
    assert_eq!(
        sim.people().polities[pi]
            .bank_site_of(law.id)
            .expect("site")
            .upkeep_h,
        asked
    );
    let mut sim = reloaded(&mut sim);
    sim.advance_minutes(60).expect("advances");
    let facts = sim
        .with_ctx_for_tests(|pop, ctx| pop.levee_facts_for_tests(ctx, owing))
        .expect("its mending to work on");
    assert_eq!(facts.bank, bank);
    assert!(
        facts.left_h > 0.0 && facts.left_h <= asked + 1e-6,
        "{facts:?}"
    );
    // Its work digs out and tamps again where water seeped through, and is given.
    let before = sim.people().polities[pi].bank_given_by(law.id, other);
    sim.with_ctx_for_tests(|pop, ctx| pop.levee_work_by_for_tests(ctx, worker, bank, asked / 4.0));
    let seg = |sim: &Sim| {
        *sim.land()
            .banks
            .iter()
            .find(|b| b.work == bank)
            .expect("its segment")
    };
    assert!(seg(&sim).piping < 0.4 && seg(&sim).piping > 0.0);
    let after = sim.people().polities[pi].bank_given_by(law.id, other);
    assert!(
        (f64::from(after - before) - asked / 4.0).abs() < 1e-2,
        "{before} {after}"
    );
    sim.with_ctx_for_tests(|pop, ctx| pop.levee_work_by_for_tests(ctx, worker, bank, asked * 2.0));
    assert_eq!(seg(&sim).piping, 0.0);
    assert!(
        !sim.people().polities[pi]
            .bank_site_of(law.id)
            .expect("site")
            .mending
    );
    assert!(sim.people().chronicle.iter().any(|e| {
        e.name
            .starts_with(&format!("{name}'s people mended the bank"))
    }));
    assert!(land_ok(&sim), "{:?}", land_problems(&sim));
    let mut sim = reloaded(&mut sim);
    // Water that pipes through it breaks it, and that is told once.
    let gave_way = |sim: &Sim| {
        sim.people()
            .chronicle
            .iter()
            .filter(|e| {
                e.name
                    .starts_with(&format!("The bank {name}'s people built"))
            })
            .count()
    };
    assert_eq!(gave_way(&sim), 0);
    {
        let land = sim.land_mut_for_tests();
        let seg = land
            .banks
            .iter_mut()
            .find(|b| b.work == bank)
            .expect("its segment");
        seg.piping = 1.0;
        seg.breached = true;
    }
    sim.advance_minutes(DAY).expect("advances");
    assert_eq!(gave_way(&sim), 1);
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| { e.name.contains("gave way: water found a way through it") })
    );
    assert_eq!(sim.people().polities[pi].banks_failed, vec![bank]);
    let mut sim = reloaded(&mut sim);
    sim.advance_minutes(DAY).expect("advances");
    assert_eq!(gave_way(&sim), 1);
    assert!(land_ok(&sim), "{:?}", land_problems(&sim));
}
