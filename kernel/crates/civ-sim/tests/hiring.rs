//! Bounded hiring, the first candidate (the owner's decisions of 2026-11-26; brief
//! `docs/briefs/bounded-hire.md` §6): a household seeking field work weighs at most
//! `firm.hire_search` distinct farm employers at a decision, its contacts first (at most
//! `firm.hire_contacts`), then employers within reach, drawn afresh by a draw keyed on the
//! household and the day; reachability is checked before a slot is spent, and at most twice the
//! budget of offers is worked out. Scenes check whom a search weighs by controlled draws (the
//! search keyed on chosen days, `hire_weighed_for_tests`) and by how often an employer is weighed
//! over many keyed days, never by one employer under every draw; and the lived decision by what
//! people then set out to do.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use civ_agents::HireOutcome;
use civ_agents::person::Target;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_land::FieldTask;
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use civ_world::{WATER_LAND, WATER_RIVER};

const PRESET: &str = "core:worldgen/river_valley";
const DAY: i64 = 24 * 60;
/// Keyed days a share is counted over.
const KEYED_DAYS: i64 = 400;

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

fn world(seed: u64, size: u32, band: u32, neighbours: Vec<u32>) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Hiring".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: size,
            band_size: band,
            neighbours,
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

/// The staged crops are sown out of season: given a timely sowing's yield, as the neighbours'
/// scenes do (`transit.rs`). A rule, not saved: set again on every load.
fn tweak(sim: &mut Sim) {
    let c = sim.rules().people.farm.crop;
    sim.rules_mut_for_tests().expect("rules").catalog.crops[c].late_sowing_loss_per_day = 0.0;
}

fn crop(sim: &Sim) -> civ_land::CropParams {
    sim.rules().catalog.crops[sim.rules().people.farm.crop].clone()
}

fn hire(sim: &Sim) -> usize {
    sim.rules()
        .catalog
        .index_of("core:activity/hired_work")
        .expect("working for hire is an activity")
}

/// The search's budget and its contacts' share, as the content sets them.
fn budget(sim: &Sim) -> (usize, usize) {
    let f = &sim.rules().people.firm;
    (f.hire_search as usize, f.hire_contacts as usize)
}

fn cell_of(sim: &Sim, p: (f32, f32)) -> usize {
    civ_agents::population::cell_of(sim.map(), p)
}

fn dry(sim: &Sim, p: (f32, f32)) -> bool {
    let (w, h) = sim.map().extent_m();
    if !(p.0 > 0.0 && p.1 > 0.0 && f64::from(p.0) < w && f64::from(p.1) < h) {
        return false;
    }
    let cell = cell_of(sim, p);
    sim.map().water[cell] == WATER_LAND && sim.nav().walkable(cell)
}

/// A place on dry ground about `at`, the `n`th tried: rings from 30 m out.
fn spot_near(sim: &Sim, at: (f32, f32), n: usize) -> (f32, f32) {
    for ring in 0..12 {
        for k in 0..12 {
            let angle = (k + n * 5) as f32 * std::f32::consts::TAU / 12.0;
            let r = 30.0 + 15.0 * ring as f32;
            let p = (at.0 + r * angle.cos(), at.1 + r * angle.sin());
            if dry(sim, p) {
                return p;
            }
        }
    }
    at
}

/// A field like the land's first for household `h`, centred at `at`, growing.
fn new_field(sim: &mut Sim, h: PermanentId, at: (f32, f32)) -> civ_land::Field {
    let template = sim.land().fields[0].clone();
    let id = sim.allocate_id_for_tests();
    let day = sim.now().day_index();
    let mut f = civ_land::Field {
        id,
        household: h,
        holder: civ_land::fields::Party::Household(h),
        lease: None,
        reserved: None,
        left_by: None,
        harvests: 0,
        stage: civ_land::FieldStage::Sown,
        sown_day: day,
        work_h: 0.0,
        sheaves_kg: 0.0,
        ..template
    };
    f.rect.x = (at.0 * 100.0) as i32 - f.rect.w / 2;
    f.rect.y = (at.1 * 100.0) as i32 - f.rect.h / 2;
    f
}

/// [`new_field`], marked out by hand.
fn add_field(sim: &mut Sim, h: PermanentId, at: (f32, f32)) -> PermanentId {
    let f = new_field(sim, h, at);
    let id = f.id;
    sim.land_mut_for_tests().fields.push(f);
    id
}

/// Household `h`'s fields moved, by hand, to lie centred at `at`.
fn move_fields(sim: &mut Sim, h: PermanentId, at: (f32, f32)) {
    for f in sim.land_mut_for_tests().fields.iter_mut() {
        if f.household == h {
            f.rect.x = (at.0 * 100.0) as i32 - f.rect.w / 2;
            f.rect.y = (at.1 * 100.0) as i32 - f.rect.h / 2;
        }
    }
}

/// Every household given a field beside its home if it has none, and every field growing, sown
/// today: nobody has field work to hire.
fn furnish(sim: &mut Sim) {
    let homes: Vec<(PermanentId, (f32, f32))> = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| (x.id, x.home))
        .collect();
    for (h, home) in homes {
        if !sim.land().fields.iter().any(|f| f.household == h) {
            let at = spot_near(sim, home, 0);
            add_field(sim, h, at);
        }
    }
    let day = sim.now().day_index();
    for f in sim.land_mut_for_tests().fields.iter_mut() {
        f.stage = civ_land::FieldStage::Sown;
        f.sown_day = day;
        f.work_h = 0.0;
        f.sheaves_kg = 0.0;
    }
    sim.people_mut_for_tests().forget_views();
}

/// The day a crop sown on it stands ripe all but a day and a half of the grace before a quarter
/// of it is lost standing, on day `day`.
fn ripe_sown(sim: &Sim, day: i64) -> i64 {
    let c = crop(sim);
    let grace = 0.25 / c.standing_loss_per_day;
    day - i64::from(c.grow_days) - (grace - 1.5).floor() as i64
}

/// Household `e`'s fields of the crop ripe ([`ripe_sown`]), tended, its grain `grain`.
fn ripe(sim: &mut Sim, e: PermanentId, grain: f64) {
    let sown = ripe_sown(sim, sim.now().day_index());
    let crop_index = sim.rules().people.farm.crop;
    for f in sim.land_mut_for_tests().fields.iter_mut() {
        if f.household == e && usize::from(f.crop) == crop_index {
            f.stage = civ_land::FieldStage::Sown;
            f.sown_day = sown;
            f.work_h = 0.0;
            f.sheaves_kg = 0.0;
            f.tended_h = 1.0e4;
        }
    }
    set_grain(sim, e, grain);
}

/// Household `e`'s fields hold 4,000 kg of sheaves between them, to be threshed.
fn sheaves(sim: &mut Sim, e: PermanentId) {
    let land = sim.land_mut_for_tests();
    let own: Vec<usize> = (0..land.fields.len())
        .filter(|&i| land.fields[i].household == e)
        .collect();
    for &i in &own {
        let f = &mut land.fields[i];
        f.stage = civ_land::FieldStage::Reaped;
        f.sheaves_kg = 4000.0 / own.len() as f32;
        f.work_h = 0.0;
    }
    sim.people_mut_for_tests().forget_views();
}

/// Household `e`'s fields lie fallow and broken, to be prepared and sown; its grain `grain`.
fn fallow(sim: &mut Sim, e: PermanentId, grain: f64) {
    for f in sim.land_mut_for_tests().fields.iter_mut() {
        if f.household == e {
            f.stage = civ_land::FieldStage::Fallow;
            f.broken = true;
            f.work_h = 0.0;
            f.sheaves_kg = 0.0;
        }
    }
    set_grain(sim, e, grain);
}

fn set_grain(sim: &mut Sim, e: PermanentId, grain: f64) {
    let good = crop(sim).good;
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == e {
            x.stores[good] = grain;
        }
    }
    sim.people_mut_for_tests().forget_views();
}

/// Households `hs`' food is gone, so a wage in grain is worth more to them than most of what
/// their time would do at home.
fn starve(sim: &mut Sim, hs: &[PermanentId]) {
    let food: Vec<usize> = sim
        .rules()
        .catalog
        .goods
        .iter()
        .enumerate()
        .filter(|(_, g)| g.kcal_per_kg > 0.0)
        .map(|(i, _)| i)
        .collect();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if hs.contains(&x.id) {
            for &g in &food {
                x.stores[g] = 0.0;
            }
        }
    }
    sim.people_mut_for_tests().forget_views();
}

fn fields_of(sim: &Sim, e: PermanentId) -> Vec<PermanentId> {
    sim.land()
        .fields
        .iter()
        .filter(|f| f.household == e)
        .map(|f| f.id)
        .collect()
}

type Searched = (Vec<(PermanentId, HireOutcome)>, usize);

/// Whom household `h`'s search weighs with its draw keyed on day `day`, and how many of the
/// first were contacts.
fn weighed(sim: &mut Sim, h: PermanentId, day: i64) -> Searched {
    sim.with_ctx_for_tests(|pop, ctx| pop.hire_weighed_for_tests(ctx, h, day))
        .expect("a search")
}

fn pool(sim: &mut Sim, h: PermanentId) -> Vec<(PermanentId, bool)> {
    sim.with_ctx_for_tests(|pop, ctx| pop.hire_pool_for_tests(ctx, h))
}

fn contacts(sim: &mut Sim, h: PermanentId) -> Vec<PermanentId> {
    sim.with_ctx_for_tests(|pop, ctx| pop.hire_contacts_for_tests(ctx, h))
}

fn job(sim: &mut Sim, h: PermanentId) -> Option<(PermanentId, Option<PermanentId>, f64)> {
    sim.with_ctx_for_tests(|pop, ctx| pop.best_job_for_tests(ctx, h))
}

fn offer(sim: &mut Sim, e: PermanentId) -> Option<(FieldTask, f64, f64)> {
    sim.with_ctx_for_tests(|pop, ctx| pop.field_hire_for_tests(ctx, e))
}

fn walk(sim: &mut Sim, h: PermanentId, at: (f32, f32)) -> Option<f64> {
    sim.with_ctx_for_tests(|pop, ctx| pop.hire_walk_for_tests(ctx, h, at))
}

fn listed(sim: &mut Sim, seeker: PermanentId, h: PermanentId) -> bool {
    pool(sim, seeker).iter().any(|&(e, _)| e == h)
}

/// A search keeps to its bounds: no employer twice, at most the budget of slots (employers
/// weighed with work or none), at most the contacts' share of them to contacts, and at most twice
/// the budget of offers worked out, retries included.
fn check(sim: &Sim, list: &[(PermanentId, HireOutcome)], from_contacts: usize) {
    let (budget, share) = budget(sim);
    let mut ids: Vec<PermanentId> = list.iter().map(|&(e, _)| e).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), list.len(), "an employer weighed twice: {list:?}");
    let slots = |l: &[(PermanentId, HireOutcome)]| {
        l.iter()
            .filter(|&&(_, o)| o != HireOutcome::PassedOver)
            .count()
    };
    assert!(slots(list) <= budget, "{} slots: {list:?}", slots(list));
    assert!(
        slots(&list[..from_contacts]) <= share,
        "contacts took {} slots",
        slots(&list[..from_contacts])
    );
    assert!(list.len() <= 2 * budget, "{} offers worked out", list.len());
}

/// `hits` of `n` near `n × p`: within four standard deviations.
fn near_share(hits: u32, n: u32, p: f64, what: &str) {
    let mean = f64::from(n) * p;
    let sd = (mean * (1.0 - p)).sqrt();
    assert!(
        (f64::from(hits) - mean).abs() <= 4.0 * sd + 1.0,
        "{what}: {hits} of {n}, about {mean:.1} expected"
    );
}

/// Over [`KEYED_DAYS`] keyed days from today, how often household `h`'s search weighs employer
/// `e` and how often finds work there; every search checked against its bounds.
fn share_of(sim: &mut Sim, h: PermanentId, e: PermanentId) -> (u32, u32) {
    let today = sim.now().day_index();
    let (mut seen, mut found) = (0, 0);
    for k in 0..KEYED_DAYS {
        let (list, c) = weighed(sim, h, today + k);
        check(sim, &list, c);
        if let Some(&(_, o)) = list.iter().find(|&&(x, _)| x == e) {
            seen += 1;
            if o == HireOutcome::Found {
                found += 1;
            }
        }
    }
    (seen, found)
}

/// The share of searches a seeker with no contacts among the employers weighs a given one in,
/// where every one it weighs spends a slot: its budget over the employers it may weigh (its own
/// household left out); and their number.
fn explore_share(sim: &mut Sim, seeker: PermanentId) -> (f64, usize) {
    let (budget, _) = budget(sim);
    let others = pool(sim, seeker)
        .iter()
        .filter(|&&(e, _)| e != seeker)
        .count();
    ((budget as f64 / others as f64).min(1.0), others)
}

/// The households of `hs` with someone who has set out to work for hire on one of `fields`.
fn set_out_for(sim: &Sim, hs: &[PermanentId], fields: &[PermanentId]) -> Vec<PermanentId> {
    let hire = hire(sim);
    let mut out: Vec<PermanentId> = sim
        .people()
        .people
        .iter()
        .filter(|(_, p)| hs.contains(&p.household))
        .filter(|(_, p)| {
            p.receipts.iter().any(|r| {
                usize::from(r.chosen.def) == hire
                    && matches!(r.chosen.target, Target::Field(f) if fields.contains(&f))
            })
        })
        .map(|(_, p)| p.household)
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

fn same(a: &Sim, b: &Sim) {
    let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
}

/// `sim` saved now and loaded, its rule set again.
fn reloaded(sim: &mut Sim) -> Sim {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "hire").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    tweak(&mut loaded);
    loaded
}

/// Saves `sim`, loads it and checks every section alike; then lives both `minutes` and checks
/// again.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let mut loaded = reloaded(sim);
    same(sim, &loaded);
    sim.advance_minutes(minutes).expect("advances");
    loaded.advance_minutes(minutes).expect("advances");
    same(sim, &loaded);
}

/// The village most scenes share: world 3's founding band, joined by fourteen families on its
/// first day, lived to the last week of the window for preparing ground; every household given a
/// field beside its home if it has none and every field growing; saved once, loaded by each
/// scene. With the founding band's households and the families', by id.
struct Village {
    _dir: tempfile::TempDir,
    path: PathBuf,
    band: Vec<PermanentId>,
    settlers: Vec<PermanentId>,
}

fn village() -> (Sim, &'static Village) {
    static VILLAGE: OnceLock<Village> = OnceLock::new();
    let v = VILLAGE.get_or_init(|| {
        let mut sim = world(3, 512, 30, Vec::new());
        let mut band: Vec<PermanentId> =
            sim.people().households.iter().map(|(_, x)| x.id).collect();
        band.sort_unstable();
        sim.send_families_to_hearth(14);
        let c = crop(&sim);
        let left = |day: i64| {
            i64::from(c.sow_until_day) - day.rem_euclid(civ_core::time::DAYS_PER_YEAR) + 1
        };
        for _ in 0..400 {
            sim.advance_minutes(DAY).expect("advances");
            let day = sim.now().day_index();
            if c.can_prepare(day) && (3..=8).contains(&left(day)) {
                break;
            }
        }
        assert!(c.can_prepare(sim.now().day_index()), "the window is open");
        band.retain(|&h| sim.people().household(h).is_some());
        let mut settlers: Vec<PermanentId> = sim
            .people()
            .households
            .iter()
            .map(|(_, x)| x.id)
            .filter(|h| !band.contains(h))
            .collect();
        settlers.sort_unstable();
        assert!(settlers.len() >= 12, "{} families settled", settlers.len());
        furnish(&mut sim);
        let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
        let saves =
            commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
        let saved = persist::save(
            &mut sim,
            &saves,
            commons_persist::SaveKind::Manual,
            "village",
        )
        .expect("saves");
        Village {
            path: saved.path.clone(),
            _dir: dir,
            band,
            settlers,
        }
    });
    let mut sim = persist::load(&v.path, content()).expect("loads");
    tweak(&mut sim);
    (sim, v)
}

/// A family come to the village's hearth now, its food gone: a newcomer with no kin or ties
/// among the village's employers, and no field.
fn newcomer(sim: &mut Sim) -> PermanentId {
    let hearth = sim.land().settlements[0].hearth_m;
    let h = sim.spawn_family(hearth).expect("a family comes").household;
    starve(sim, &[h]);
    h
}

/// Discovery by an unconnected newcomer (§6, "unconnected newcomer"): a family come to the
/// village has no kin or ties among its employers, so every slot of its search goes to the
/// employers within reach. Among twenty, one short of reapers is weighed in about the budget's
/// share of its keyed searches (never in all, never in none) and found whenever weighed, and its
/// reaping is the job the newcomer's decision finds exactly when today's keyed search weighs it.
/// Each search keeps its bounds. A search that never explored would never find it.
#[test]
fn a_newcomer_without_kin_or_ties_finds_field_work_by_search() {
    let (mut sim, v) = village();
    let (budget, _) = budget(&sim);
    let employer = v.band[0];
    ripe(&mut sim, employer, 400.0);
    let seeker = newcomer(&mut sim);
    let o = offer(&mut sim, employer);
    assert!(
        o.is_some_and(|o| o.0 == FieldTask::Reap && o.1 >= 0.5),
        "the employer is short of reapers: {o:?}"
    );
    assert!(
        contacts(&mut sim, seeker).is_empty(),
        "a newcomer has no kin or ties among the employers"
    );
    let all = pool(&mut sim, seeker);
    assert!(all.iter().any(|&(e, _)| e == employer), "within reach");
    for &(e, _) in &all {
        if e != employer {
            assert!(offer(&mut sim, e).is_none(), "{e} has no work to hire");
        }
    }
    let (p, others) = explore_share(&mut sim, seeker);
    assert!(
        others > 2 * budget,
        "{others} employers, more than the search weighs"
    );
    let (seen, found) = share_of(&mut sim, seeker, employer);
    assert_eq!(seen, found, "found whenever weighed");
    near_share(
        found,
        KEYED_DAYS as u32,
        p,
        "searches weighing the employer",
    );
    assert!(found > 0 && found < KEYED_DAYS as u32);
    // The decision's own search is today's keyed one.
    let today = sim.now().day_index();
    let (list, _) = weighed(&mut sim, seeker, today);
    let job = job(&mut sim, seeker);
    let fields = fields_of(&sim, employer);
    if list.iter().any(|&(e, _)| e == employer) {
        assert!(
            job.is_some_and(|(e, f, _)| e == employer && f.is_some_and(|f| fields.contains(&f))),
            "{job:?}"
        );
    } else {
        assert!(job.is_none(), "nothing else to hire: {job:?}");
    }
}

/// Field work of each kind is found by search (§6, "reaping, threshing and sowing separately"):
/// one employer among twenty short of hands for the work `stage` sets it short of (`task`) is
/// weighed by a newcomer in about the budget's share of keyed searches and found whenever weighed;
/// and, with every household's food gone, some of those whose searches weigh it (by contact or
/// exploration) go and do the work for hire within three days and are paid in grain, while none
/// goes whose search that day does not weigh it.
fn found_by_search(task: FieldTask, stage: impl Fn(&mut Sim, PermanentId)) {
    let (mut sim, v) = village();
    let employer = v.band[0];
    stage(&mut sim, employer);
    let o = offer(&mut sim, employer);
    assert!(
        o.is_some_and(|o| o.0 == task && o.1 >= 0.5),
        "short of hands for {task:?}: {o:?}"
    );
    let seeker = newcomer(&mut sim);
    assert!(
        contacts(&mut sim, seeker).is_empty(),
        "a newcomer has no contacts"
    );
    let (p, _) = explore_share(&mut sim, seeker);
    let (seen, found) = share_of(&mut sim, seeker, employer);
    assert_eq!(seen, found);
    near_share(found, KEYED_DAYS as u32, p, "searches weighing it");
    // Every household's food gone; over up to three days, some whose searches weigh it go and
    // work for it; each that goes had it in its search that day, as it stood the hour before.
    let mut everyone: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x.id)
        .filter(|&h| h != employer)
        .collect();
    everyone.sort_unstable();
    starve(&mut sim, &everyone);
    let weighing_now = |sim: &mut Sim| -> Vec<PermanentId> {
        let day = sim.now().day_index();
        everyone
            .iter()
            .copied()
            .filter(|&h| weighed(sim, h, day).0.iter().any(|&(e, _)| e == employer))
            .collect()
    };
    let first = weighing_now(&mut sim);
    assert!(
        !first.is_empty() && first.len() < everyone.len(),
        "some households weigh it today, not all: {} of {}",
        first.len(),
        everyone.len()
    );
    let fields = fields_of(&sim, employer);
    let grain = crop(&sim).good;
    let wages = sim.people().transfers.get(civ_agents::Channel::Wage, grain);
    let tally = sim.people().hire_tally();
    let mut went: Vec<PermanentId> = Vec::new();
    for _ in 0..3 * 24 {
        let weighing = weighing_now(&mut sim);
        sim.advance_minutes(60).expect("advances");
        for h in set_out_for(&sim, &everyone, &fields) {
            if !went.contains(&h) {
                assert!(
                    weighing.contains(&h),
                    "{h} went without its search weighing it"
                );
                went.push(h);
            }
        }
        if !went.is_empty() && sim.people().transfers.get(civ_agents::Channel::Wage, grain) > wages
        {
            break;
        }
    }
    assert!(
        !went.is_empty(),
        "a household weighing it went to work for hire"
    );
    assert!(
        sim.people().transfers.get(civ_agents::Channel::Wage, grain) > wages,
        "paid in grain"
    );
    // The lived searches kept their bounds.
    let t = sim.people().hire_tally();
    assert!(t.decisions > tally.decisions, "searches were made");
    assert!(t.most_offers <= 2 * budget(&sim).0 as u64, "{t:?}");
}

#[test]
fn a_short_handed_reaper_among_many_employers_is_found_by_search() {
    found_by_search(FieldTask::Reap, |sim, e| ripe(sim, e, 400.0));
}

#[test]
fn a_short_handed_thresher_among_many_employers_is_found_by_search() {
    found_by_search(FieldTask::Thresh, sheaves);
}

#[test]
fn a_short_handed_sower_among_many_employers_is_found_by_search() {
    found_by_search(FieldTask::Sow, |sim, e| fallow(sim, e, 5000.0));
}

/// Changing vacancies (§6): what an employer offers is read when it is weighed, never kept with
/// whom a search may weigh. A seeker's keyed searches find one employer short of reapers whenever
/// they weigh it and nothing at another; once the first's reaping is covered and the second falls
/// short, the same keyed searches, over the same kept list of employers (not built again), weigh
/// the same employers and find the second's. A list that kept who was hiring when it was built
/// would never find the second.
#[test]
fn seekers_turn_to_a_vacancy_opened_once_another_is_covered() {
    let (mut sim, v) = village();
    let (first, second) = (v.band[0], v.band[1]);
    let seeker = newcomer(&mut sim);
    ripe(&mut sim, first, 400.0);
    set_grain(&mut sim, second, 400.0);
    assert!(offer(&mut sim, first).is_some() && offer(&mut sim, second).is_none());
    let today = sim.now().day_index();
    let before: Vec<Searched> = (0..60)
        .map(|k| weighed(&mut sim, seeker, today + k))
        .collect();
    let builds = sim.people().hire_tally().pool_builds;
    // The first's reaping is covered and the second's crop has stood ripe: changed in place, as
    // a day's work changes fields, with nothing derived let go.
    let c = sim.rules().people.farm.crop;
    let sown = ripe_sown(&sim, today);
    sim.with_ctx_for_tests(|_, ctx| {
        for f in ctx.land.fields.iter_mut() {
            if f.household == first {
                f.sown_day = today;
            } else if f.household == second && usize::from(f.crop) == c {
                f.sown_day = sown;
                f.work_h = 0.0;
                f.tended_h = 1.0e4;
            }
        }
    });
    assert!(offer(&mut sim, first).is_none() && offer(&mut sim, second).is_some());
    let after: Vec<Searched> = (0..60)
        .map(|k| weighed(&mut sim, seeker, today + k))
        .collect();
    assert_eq!(
        sim.people().hire_tally().pool_builds,
        builds,
        "the kept list of employers was not built again"
    );
    let (mut first_found, mut second_found) = (0, 0);
    for ((b, _), (a, _)) in before.iter().zip(&after) {
        let ids = |l: &[(PermanentId, HireOutcome)]| l.iter().map(|&(e, _)| e).collect::<Vec<_>>();
        assert_eq!(ids(b), ids(a), "the same employers weighed");
        for &(e, o) in b {
            assert_eq!(o == HireOutcome::Found, e == first, "before: {e} {o:?}");
            first_found += u32::from(e == first);
        }
        for &(e, o) in a {
            assert_eq!(o == HireOutcome::Found, e == second, "after: {e} {o:?}");
            second_found += u32::from(e == second);
        }
    }
    assert!(
        first_found > 0 && second_found > 0,
        "{first_found} {second_found}"
    );
}

/// Several fields, one slot (§6): an employer working six fields is listed once among those a
/// search may weigh and weighed at most once a search, in about the same share of keyed searches
/// as an employer with one. A list holding an employer once a field would weigh it about six
/// times as often.
#[test]
fn an_employer_with_many_fields_takes_one_slot_of_the_search() {
    let (mut sim, v) = village();
    let many = v.band[1];
    let one = *v
        .settlers
        .iter()
        .find(|&&h| fields_of(&sim, h).len() == 1)
        .expect("a household with one field");
    let seeker = newcomer(&mut sim);
    let home = sim.people().household(many).expect("its household").home;
    while fields_of(&sim, many).len() < 6 {
        let n = fields_of(&sim, many).len();
        let at = spot_near(&sim, home, n);
        add_field(&mut sim, many, at);
    }
    let all = pool(&mut sim, seeker);
    assert_eq!(
        all.iter().filter(|&&(e, _)| e == many).count(),
        1,
        "listed once"
    );
    let (p, _) = explore_share(&mut sim, seeker);
    let (seen_many, _) = share_of(&mut sim, seeker, many);
    let (seen_one, _) = share_of(&mut sim, seeker, one);
    near_share(
        seen_many,
        KEYED_DAYS as u32,
        p,
        "the employer with six fields",
    );
    near_share(seen_one, KEYED_DAYS as u32, p, "the employer with one");
}

/// Payment capacity (§6): an employer short of reapers that holds no grain to pay them offers
/// nothing: weighed (a slot spent), never hired. Holding grain, the same keyed searches find it.
/// Were its offer not capped by the grain it holds, it would be found with none.
#[test]
fn an_employer_that_cannot_pay_is_weighed_and_not_hired() {
    let (mut sim, v) = village();
    let (budget, _) = budget(&sim);
    let employer = v.band[0];
    let seeker = newcomer(&mut sim);
    ripe(&mut sim, employer, 0.0);
    assert!(
        offer(&mut sim, employer).is_none(),
        "nothing it can pay for"
    );
    let today = sim.now().day_index();
    let mut weighed_days = Vec::new();
    for k in 0..120 {
        let (list, c) = weighed(&mut sim, seeker, today + k);
        check(&sim, &list, c);
        assert_eq!(
            list.len(),
            budget,
            "every employer weighed spends a slot: {list:?}"
        );
        if let Some(&(_, o)) = list.iter().find(|&&(e, _)| e == employer) {
            assert_eq!(o, HireOutcome::NoWork, "weighed, not hired");
            weighed_days.push(k);
        }
    }
    assert!(!weighed_days.is_empty(), "weighed on some keyed days");
    assert!(job(&mut sim, seeker).is_none(), "nothing to hire");
    set_grain(&mut sim, employer, 400.0);
    assert!(offer(&mut sim, employer).is_some(), "grain to pay with");
    for &k in &weighed_days {
        let (list, _) = weighed(&mut sim, seeker, today + k);
        assert!(
            list.iter()
                .any(|&(e, o)| e == employer && o == HireOutcome::Found),
            "found on keyed day {k}"
        );
    }
}

/// Competition with the household's own production (§6): a search never weighs the seeker's own
/// household, so a household whose own crop stands ripe and short of hands reaps it as its own
/// work, while another employer short of reapers is in its searches. Were its own household among
/// those weighed, its people would set out to "work for hire" on their own field.
#[test]
fn a_seeker_whose_own_crop_stands_ripe_reaps_its_own_before_hire() {
    let (mut sim, v) = village();
    let seeker = *v
        .settlers
        .iter()
        .min_by_key(|&&h| (sim.people().household(h).map_or(0, |x| x.members.len()), h))
        .expect("a settler");
    let other = v.band[0];
    let home = sim.people().household(seeker).expect("its household").home;
    while fields_of(&sim, seeker).len() < 10 {
        let n = fields_of(&sim, seeker).len();
        let at = spot_near(&sim, home, n);
        add_field(&mut sim, seeker, at);
    }
    ripe(&mut sim, seeker, 400.0);
    ripe(&mut sim, other, 400.0);
    let o = offer(&mut sim, seeker);
    assert!(
        o.is_some_and(|o| o.0 == FieldTask::Reap),
        "its own crop needs more hands than it has: {o:?}"
    );
    let today = sim.now().day_index();
    let mut other_found = 0;
    for k in 0..KEYED_DAYS {
        let (list, c) = weighed(&mut sim, seeker, today + k);
        check(&sim, &list, c);
        assert!(list.iter().all(|&(e, _)| e != seeker), "never itself");
        other_found += u32::from(list.iter().any(|&(e, _)| e == other));
    }
    assert!(other_found > 0, "the other employer is in its searches");
    let own = fields_of(&sim, seeker);
    let worked = |sim: &Sim| -> f64 {
        sim.land()
            .fields
            .iter()
            .filter(|f| own.contains(&f.id))
            .map(|f| f64::from(f.work_h) + f64::from(f.harvests) * 1e6)
            .sum()
    };
    let before = worked(&sim);
    for _ in 0..16 {
        sim.advance_minutes(60).expect("advances");
    }
    assert!(worked(&sim) > before, "its own crop was reaped");
    assert!(
        set_out_for(&sim, &[seeker], &own).is_empty(),
        "nobody of it set out to work its own field for hire"
    );
}

/// Saves mid-search (§6, "saves and modes"): with employers short of reapers, threshers and
/// sowers and many seekers, a Detailed world saved at 09:30, as people weigh and take field work
/// for hire, reloads to the same searches and goes on alike, every section, four hours on; and so
/// does a save at a household-resolved midnight and at an Accelerated midnight, each lived two days
/// on beside the unsaved world.
#[test]
fn a_detailed_save_mid_morning_at_the_harvest_lives_on_alike() {
    let (mut sim, v) = village();
    ripe(&mut sim, v.band[0], 400.0);
    sheaves(&mut sim, v.band[1]);
    fallow(&mut sim, v.band[2], 5000.0);
    starve(&mut sim, &v.settlers);
    let into = sim.now().minute_of_day();
    sim.advance_minutes(9 * 60 + 30 - into).expect("advances");
    let today = sim.now().day_index();
    let searches = |sim: &mut Sim| -> Vec<Searched> {
        v.settlers.iter().map(|&h| weighed(sim, h, today)).collect()
    };
    let mine = searches(&mut sim);
    let mut loaded = reloaded(&mut sim);
    assert_eq!(
        searches(&mut loaded),
        mine,
        "the same searches after a load"
    );
    assert!(
        mine.iter()
            .flat_map(|(l, _)| l)
            .any(|&(_, o)| o == HireOutcome::Found),
        "work found mid-search"
    );
    let tally = sim.people().hire_tally();
    saves_and_goes_on_alike(&mut sim, 4 * 60);
    assert!(
        sim.people().hire_tally().decisions > tally.decisions,
        "searches went on"
    );
    // A household-resolved day: saved at the midnight it begins.
    let s = sim.land().settlements[0].id;
    sim.set_household_resolved([s]);
    let to_midnight = DAY - sim.now().minute_of_day();
    sim.advance_minutes(to_midnight).expect("advances");
    assert_eq!(sim.household_resolved().len(), 1);
    let mut loaded = reloaded(&mut sim);
    loaded.set_household_resolved([s]);
    same(&sim, &loaded);
    sim.advance_minutes(2 * DAY).expect("advances");
    loaded.advance_minutes(2 * DAY).expect("advances");
    same(&sim, &loaded);
    // An Accelerated midnight, Detailed resolution.
    sim.set_household_resolved([]);
    sim.set_speed(SPEED_MAX).expect("Max");
    sim.advance_minutes(DAY).expect("advances");
    let mut loaded = reloaded(&mut sim);
    loaded.set_speed(SPEED_MAX).expect("Max");
    same(&sim, &loaded);
    sim.advance_minutes(2 * DAY).expect("advances");
    loaded.advance_minutes(2 * DAY).expect("advances");
    same(&sim, &loaded);
}

/// The world of a river too big to wade: world 5's village, twenty families sent, every
/// household given a field beside its home and every field growing; with its founding band's
/// households and the newcomers', and the dry land within 2 km of the hearth (inside the work's
/// straight line) that no walk reaches, across the river, nearest first.
#[allow(clippy::type_complexity)]
fn across_water() -> (Sim, Vec<PermanentId>, Vec<PermanentId>, Vec<(f32, f32)>) {
    let mut sim = world(5, 512, 30, Vec::new());
    tweak(&mut sim);
    sim.advance_minutes(DAY).expect("advances");
    let mut band: Vec<PermanentId> = sim.people().households.iter().map(|(_, x)| x.id).collect();
    band.sort_unstable();
    sim.send_families_to_hearth(20);
    sim.advance_minutes(60).expect("advances");
    let mut newcomers: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x.id)
        .filter(|h| !band.contains(h))
        .collect();
    newcomers.sort_unstable();
    assert!(newcomers.len() >= 16, "{} families came", newcomers.len());
    furnish(&mut sim);
    let hearth = sim.land().settlements[0].hearth_m;
    let scan = sim.with_ctx_for_tests(|pop, ctx| pop.hire_scan_m_for_tests(ctx)) as f32;
    let h = band[0];
    let mut far = Vec::new();
    let (w, ht) = sim.map().extent_m();
    let mut y = 20.0f32;
    while f64::from(y) < ht {
        let mut x = 20.0f32;
        while f64::from(x) < w {
            let p = (x, y);
            if (p.0 - hearth.0).hypot(p.1 - hearth.1) <= scan.min(2000.0)
                && dry(&sim, p)
                && walk(&mut sim, h, p).is_none()
            {
                far.push(p);
            }
            x += 80.0;
        }
        y += 80.0;
    }
    let d = |p: &(f32, f32)| (p.0 - hearth.0).hypot(p.1 - hearth.1);
    far.sort_by(|a, b| d(a).total_cmp(&d(b)));
    assert!(
        far.len() >= 20,
        "dry land no walk reaches near the hearth: {}",
        far.len()
    );
    // Across water: the straight line from the hearth to the nearest crosses a river.
    let p = far[0];
    let crosses = (0..400).any(|i| {
        let t = i as f32 / 400.0;
        let q = (
            hearth.0 + (p.0 - hearth.0) * t,
            hearth.1 + (p.1 - hearth.1) * t,
        );
        sim.map().water[cell_of(&sim, q)] == WATER_RIVER
    });
    assert!(crosses, "the far bank lies across a river");
    (sim, band, newcomers, far)
}

/// Inaccessible nearby work takes no slot (§6, a water barrier): twelve households whose homes and
/// fields lie across a river too big to wade, within the work's straight line of the hearth, are
/// not among those a newcomer's search may weigh; the employers it can walk to number no more than
/// the search's budget, so every keyed search weighs every one of them and finds the one short of
/// reapers. Were the straight line taken for reach, the idle households across the water would
/// take slots and crowd it out of some searches.
#[test]
fn fields_across_water_with_no_crossing_take_no_slot_from_reachable_work() {
    let (mut sim, band, newcomers, far) = across_water();
    let (budget, _) = budget(&sim);
    let seeker = newcomers[0];
    // All but the seeker and as many as leave the budget within reach move across.
    let stay = budget.saturating_sub(band.len()).max(1);
    let across: Vec<PermanentId> = newcomers[stay..].to_vec();
    assert!(across.len() >= 12, "{} across", across.len());
    for (i, &h) in across.iter().enumerate() {
        let at = far[i * far.len() / across.len()];
        sim.with_ctx_for_tests(|pop, _| pop.move_home_for_tests(h, at));
        move_fields(&mut sim, h, at);
    }
    let employer = band[0];
    ripe(&mut sim, employer, 400.0);
    starve(&mut sim, &[seeker]);
    assert!(offer(&mut sim, employer).is_some());
    let all = pool(&mut sim, seeker);
    for &h in &across {
        assert!(
            all.iter().all(|&(e, _)| e != h),
            "{h} across the water is not listed"
        );
    }
    let near: Vec<PermanentId> = all
        .iter()
        .map(|&(e, _)| e)
        .filter(|&e| e != seeker)
        .collect();
    assert!(
        near.len() <= budget && near.contains(&employer),
        "{} employers within reach, the budget {budget}",
        near.len()
    );
    assert!(
        across.len() + near.len() > budget,
        "the straight line holds more than the budget"
    );
    let (seen, found) = share_of(&mut sim, seeker, employer);
    assert_eq!(
        (seen, found),
        (KEYED_DAYS as u32, KEYED_DAYS as u32),
        "every search weighs and finds it"
    );
}

/// At most twice the budget of offers (§6, "search cost"; the owner's hard limit of sixteen
/// offer evaluations, retries included): households of the village whose crops stand ripe across
/// the river, short of reapers, offer work no walk reaches, and each is passed over without a
/// slot. A search goes on past them to the employers within reach and stops at twice its budget
/// of offers worked out; the one within reach short of reapers is found in the searches that reach
/// it within that limit. Without the limit a search would work out every offer.
#[test]
fn a_search_works_out_no_more_than_twice_its_budget_of_offers() {
    let (mut sim, band, newcomers, far) = across_water();
    let (budget, _) = budget(&sim);
    let seeker = newcomers[0];
    let mut cut_off: Vec<PermanentId> = newcomers[1..].to_vec();
    for (i, &h) in cut_off.iter().enumerate() {
        let at = far[i * far.len() / cut_off.len()];
        for n in 0..3 {
            let p = spot_near(&sim, at, n);
            add_field(&mut sim, h, p);
        }
        move_fields(&mut sim, h, at);
        ripe(&mut sim, h, 400.0);
    }
    cut_off.retain(|&h| offer(&mut sim, h).is_some());
    assert!(
        cut_off.len() > 2 * budget,
        "{} short of reapers across the water",
        cut_off.len()
    );
    let employer = band[0];
    ripe(&mut sim, employer, 400.0);
    assert!(offer(&mut sim, employer).is_some());
    let all = pool(&mut sim, seeker);
    for &h in &cut_off {
        assert!(
            all.iter().any(|&(e, _)| e == h),
            "{h}'s home is within reach"
        );
    }
    let today = sim.now().day_index();
    let (mut capped, mut found) = (0, 0);
    for k in 0..KEYED_DAYS {
        let (list, c) = weighed(&mut sim, seeker, today + k);
        check(&sim, &list, c);
        for &(e, o) in &list {
            if cut_off.contains(&e) {
                assert_eq!(o, HireOutcome::PassedOver, "out of reach, no slot");
            }
        }
        capped += u32::from(list.len() == 2 * budget);
        found += u32::from(
            list.iter()
                .any(|&(e, o)| e == employer && o == HireOutcome::Found),
        );
    }
    assert!(capped > 0, "some searches stop at the limit");
    assert!(
        found > 0 && found < KEYED_DAYS as u32,
        "found in {found} searches"
    );
    let t = sim.people().hire_tally();
    assert!(
        t.most_offers <= 2 * budget as u64 && t.passed_over > 0,
        "{t:?}"
    );
}

/// The kept list of employers follows each input it reads (§6, "kept pool"; the brief's §4.2a):
/// a field changing hands, a new field, an employer's home moved across the water and back, a
/// household founded and one gone. After each, the list a newcomer's search reads is the one
/// built afresh (the debug check at every read), and a world saved and loaded at that hour reads
/// the same list, the same searches and finds the same job.
#[test]
fn the_kept_list_of_employers_follows_each_change_as_a_loaded_world_reads_it() {
    let (mut sim, band, newcomers, far) = across_water();
    let seeker = newcomers[0];
    ripe(&mut sim, band[0], 400.0);
    let today = sim.now().day_index();
    let alike = |sim: &mut Sim, what: &str| {
        let mut loaded = reloaded(sim);
        assert_eq!(
            pool(sim, seeker),
            pool(&mut loaded, seeker),
            "{what}: the list"
        );
        for k in 0..6 {
            assert_eq!(
                weighed(sim, seeker, today + k),
                weighed(&mut loaded, seeker, today + k),
                "{what}: keyed day {k}"
            );
        }
        assert_eq!(
            job(sim, seeker),
            job(&mut loaded, seeker),
            "{what}: the job"
        );
    };
    alike(&mut sim, "as it stands");
    // A field changes hands, its old household left with none.
    let (from, to) = (newcomers[1], newcomers[2]);
    assert!(listed(&mut sim, seeker, from));
    sim.with_ctx_for_tests(|pop, ctx| pop.move_fields_for_tests(ctx.land, from, to));
    assert!(
        !listed(&mut sim, seeker, from),
        "with no field it hires nobody"
    );
    alike(&mut sim, "a field changed hands");
    // A new field is marked out for it.
    let home = sim.people().household(from).expect("its household").home;
    let at = spot_near(&sim, home, 3);
    let f = new_field(&mut sim, from, at);
    sim.with_ctx_for_tests(|_, ctx| ctx.land.fields.push(f));
    assert!(listed(&mut sim, seeker, from), "a new field");
    alike(&mut sim, "a new field");
    // An employer's field across the water, by hand (what is derived let go), then its home, as
    // a move counts it: no longer within reach.
    let mover = newcomers[3];
    let there = far[0];
    move_fields(&mut sim, mover, there);
    assert!(
        listed(&mut sim, seeker, mover),
        "its home is within reach yet"
    );
    sim.with_ctx_for_tests(|pop, _| pop.move_home_for_tests(mover, there));
    assert!(
        !listed(&mut sim, seeker, mover),
        "its home moved across the water"
    );
    alike(&mut sim, "a home moved away");
    let hearth = sim.land().settlements[0].hearth_m;
    let back = spot_near(&sim, hearth, 2);
    sim.with_ctx_for_tests(|pop, _| pop.move_home_for_tests(mover, back));
    assert!(listed(&mut sim, seeker, mover), "its home moved back");
    alike(&mut sim, "a home moved back");
    // A household founded: a family comes, and is given a field.
    let came = sim.spawn_family(hearth).expect("a family comes").household;
    assert!(!listed(&mut sim, seeker, came), "no field yet");
    let home = sim.people().household(came).expect("its household").home;
    let at = spot_near(&sim, home, 1);
    let f = new_field(&mut sim, came, at);
    sim.with_ctx_for_tests(|_, ctx| ctx.land.fields.push(f));
    assert!(listed(&mut sim, seeker, came), "founded, with a field");
    alike(&mut sim, "a household founded");
    // A household gone: all its people die.
    let gone = newcomers[4];
    let members = sim
        .people()
        .household(gone)
        .expect("its household")
        .members
        .clone();
    for m in members {
        sim.with_ctx_for_tests(|pop, ctx| pop.die_for_tests(ctx, m));
    }
    assert!(!listed(&mut sim, seeker, gone), "gone");
    alike(&mut sim, "a household gone");
}

/// A ford closed by a flood (§6, inaccessible nearby work; the kept list's travel field): world
/// 4's hearth has streams that can be waded within a walk (`crossings.rs`), and dry ground that
/// only a ford reaches. An employer whose home and fields lie there, short of reapers, is weighed in its share of a newcomer's keyed
/// searches while the ford is open; when the stream swells past wading no walk reaches it, and it
/// leaves the list and every search (the list built again for the walking grid's new revision,
/// the debug check against one built afresh at every read); when the flood falls it comes back.
#[test]
fn a_ford_closed_by_a_flood_takes_the_far_bank_out_of_the_search_until_it_falls() {
    let mut sim = world(4, 768, 30, Vec::new());
    tweak(&mut sim);
    sim.advance_minutes(DAY).expect("lives");
    let mut band: Vec<PermanentId> = sim.people().households.iter().map(|(_, x)| x.id).collect();
    band.sort_unstable();
    sim.send_families_to_hearth(3);
    sim.advance_minutes(60).expect("advances");
    furnish(&mut sim);
    let newcomer = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x.id)
        .filter(|h| !band.contains(h))
        .min()
        .expect("a newcomer");
    // Every stream that can be waded swells past wading in the flood.
    let hearth = sim.land().settlements[0].hearth_m;
    let ford_max = sim.nav().params().ford_max_discharge_m3s as f32;
    let open = sim.land().reach_discharges.clone();
    let flooded = |sim: &mut Sim, swell: bool| {
        sim.with_ctx_for_tests(|_, ctx| {
            for (q, o) in ctx.land.reach_discharges.iter_mut().zip(&open) {
                *q = if swell && *o <= ford_max {
                    ford_max * 10.0 + 1.0
                } else {
                    *o
                };
            }
            ctx.land.flood_revision += 1;
        });
    };
    // Dry ground within 1.5 km that the newcomer's walk reaches while the fords are open and no
    // longer once they are not.
    let mut reached = Vec::new();
    let mut y = hearth.1 - 1500.0;
    while y <= hearth.1 + 1500.0 {
        let mut x = hearth.0 - 1500.0;
        while x <= hearth.0 + 1500.0 {
            let p = (x, y);
            if (x - hearth.0).hypot(y - hearth.1) <= 1500.0
                && dry(&sim, p)
                && walk(&mut sim, newcomer, p).is_some()
            {
                reached.push(p);
            }
            x += 60.0;
        }
        y += 60.0;
    }
    flooded(&mut sim, true);
    let mut cut: Vec<(f32, f32)> = reached
        .into_iter()
        .filter(|&p| walk(&mut sim, newcomer, p).is_none())
        .collect();
    flooded(&mut sim, false);
    let d = |p: &(f32, f32)| (p.0 - hearth.0).hypot(p.1 - hearth.1);
    cut.sort_by(|a, b| d(a).total_cmp(&d(b)));
    let beyond = *cut
        .first()
        .expect("dry ground a flood of the fords cuts off");
    // Across a ford: the straight line from the hearth crosses a wadeable river.
    let crosses = (0..400).any(|i| {
        let t = i as f32 / 400.0;
        let q = (
            hearth.0 + (beyond.0 - hearth.0) * t,
            hearth.1 + (beyond.1 - hearth.1) * t,
        );
        sim.map().water[cell_of(&sim, q)] == WATER_RIVER
    });
    assert!(crosses, "a stream lies between");
    let employer = band[0];
    sim.with_ctx_for_tests(|pop, _| pop.move_home_for_tests(employer, beyond));
    move_fields(&mut sim, employer, beyond);
    ripe(&mut sim, employer, 400.0);
    starve(&mut sim, &[newcomer]);
    assert!(
        walk(&mut sim, newcomer, beyond).is_some(),
        "the ford is waded"
    );
    assert!(offer(&mut sim, employer).is_some());
    let (p, _) = explore_share(&mut sim, newcomer);
    let found = share_of(&mut sim, newcomer, employer).1;
    near_share(found, KEYED_DAYS as u32, p, "the ford open");
    flooded(&mut sim, true);
    assert!(
        walk(&mut sim, newcomer, beyond).is_none(),
        "the swollen stream is not waded"
    );
    assert!(
        !listed(&mut sim, newcomer, employer),
        "the far bank's employer is not listed"
    );
    assert_eq!(
        share_of(&mut sim, newcomer, employer),
        (0, 0),
        "nor weighed"
    );
    flooded(&mut sim, false);
    assert!(walk(&mut sim, newcomer, beyond).is_some(), "waded again");
    let found = share_of(&mut sim, newcomer, employer).1;
    near_share(found, KEYED_DAYS as u32, p, "the flood fallen");
}

/// Reachable neighbouring work (§6): a household of another settlement short of reapers, its field
/// within the work's walk of the hand's hearth, is among those a newcomer of the hand's
/// settlement may weigh, on the same footing as the twenty of its own: weighed in about the
/// budget's share of its keyed searches, found whenever weighed. A list of the seeker's own
/// settlement alone would never weigh it.
#[test]
fn a_neighbours_reachable_employer_is_weighed_beside_idle_home_employers() {
    let mut sim = world(3, 768, 30, vec![30]);
    tweak(&mut sim);
    for _ in 0..200 {
        if sim.land().fields.len() >= 8 {
            break;
        }
        sim.advance_minutes(DAY).expect("advances");
    }
    let c = sim.rules().people.farm.crop;
    // The employer: the household with the most fields of the crop, as the neighbours' scenes
    // of `transit.rs` take it.
    let mut count: std::collections::BTreeMap<PermanentId, usize> = Default::default();
    for f in sim
        .land()
        .fields
        .iter()
        .filter(|f| usize::from(f.crop) == c)
    {
        *count.entry(f.household).or_default() += 1;
    }
    let employer = count
        .iter()
        .filter(|(h, _)| {
            sim.people()
                .household(**h)
                .is_some_and(|x| x.settlement.is_some())
        })
        .max_by_key(|(h, n)| (**n, std::cmp::Reverse(**h)))
        .map(|(h, _)| *h)
        .expect("a household with fields");
    let home_of = sim.people().household(employer).and_then(|x| x.settlement);
    let (a, b) = {
        let s = &sim.land().settlements;
        let a = s.iter().find(|x| Some(x.id) == home_of).expect("`a`");
        let b = s.iter().find(|x| Some(x.id) != home_of).expect("`b`");
        (a.hearth_m, (b.id, b.hearth_m))
    };
    let came = sim.spawn_families(b.1, 14).expect("families come to `b`");
    assert!(came.iter().all(|s| s.settlement == b.0), "they join `b`");
    sim.advance_minutes(60).expect("advances");
    furnish(&mut sim);
    let seeker = came
        .iter()
        .map(|s| s.household)
        .min()
        .expect("a newcomer of `b`");
    ripe(&mut sim, employer, 400.0);
    starve(&mut sim, &[seeker]);
    // Its field ripe longest 400 m from `b`'s hearth toward `a`.
    let (dx, dy) = (a.0 - b.1.0, a.1 - b.1.1);
    let d = dx.hypot(dy).max(1.0);
    let at = (b.1.0 + dx / d * 400.0, b.1.1 + dy / d * 400.0);
    let first = fields_of(&sim, employer)[0];
    for f in sim.land_mut_for_tests().fields.iter_mut() {
        if f.id == first {
            f.rect.x = (at.0 * 100.0) as i32 - f.rect.w / 2;
            f.rect.y = (at.1 * 100.0) as i32 - f.rect.h / 2;
            f.sown_day -= 1;
        }
    }
    assert!(
        offer(&mut sim, employer).is_some_and(|o| o.0 == FieldTask::Reap),
        "short of reapers"
    );
    let w = walk(&mut sim, seeker, at);
    assert!(w.is_some_and(|w| w <= 30.0), "within the walk: {w:?}");
    assert!(
        contacts(&mut sim, seeker).is_empty(),
        "a newcomer: no contacts"
    );
    let all = pool(&mut sim, seeker);
    assert!(
        all.contains(&(employer, true)),
        "listed, of another settlement"
    );
    let (p, others) = explore_share(&mut sim, seeker);
    assert!(others > 2 * budget(&sim).0, "{others} employers");
    let (seen, found) = share_of(&mut sim, seeker, employer);
    assert_eq!(seen, found, "found whenever weighed");
    near_share(
        found,
        KEYED_DAYS as u32,
        p,
        "searches weighing the neighbour",
    );
}

/// The nineteenth coverage pass (§9): a household of another settlement is listed among those a
/// newcomer may weigh by a field within the work's walk of the hand's hearth, but the work it
/// offers is on its other field, ripe longest, by its own settlement and beyond the walk. Whenever
/// it is weighed it is passed over, never found and never spending a slot, lest a far one hide a
/// near one (§6). Were another settlement's offer taken at any walk, the hand would be sent far
/// from its own hearth for the work.
#[test]
fn a_neighbours_employer_listed_by_a_near_field_is_passed_over_for_work_beyond_the_walk() {
    let mut sim = world(3, 768, 30, vec![30]);
    tweak(&mut sim);
    for _ in 0..200 {
        if sim.land().fields.len() >= 8 {
            break;
        }
        sim.advance_minutes(DAY).expect("advances");
    }
    let c = sim.rules().people.farm.crop;
    let mut count: std::collections::BTreeMap<PermanentId, usize> = Default::default();
    for f in sim
        .land()
        .fields
        .iter()
        .filter(|f| usize::from(f.crop) == c)
    {
        *count.entry(f.household).or_default() += 1;
    }
    let employer = count
        .iter()
        .filter(|(h, n)| {
            **n >= 2
                && sim
                    .people()
                    .household(**h)
                    .is_some_and(|x| x.settlement.is_some())
        })
        .max_by_key(|(h, n)| (**n, std::cmp::Reverse(**h)))
        .map(|(h, _)| *h)
        .expect("a household with two fields");
    let home_of = sim.people().household(employer).and_then(|x| x.settlement);
    let (a, b) = {
        let s = &sim.land().settlements;
        let a = s.iter().find(|x| Some(x.id) == home_of).expect("`a`");
        let b = s.iter().find(|x| Some(x.id) != home_of).expect("`b`");
        (a.hearth_m, (b.id, b.hearth_m))
    };
    let came = sim.spawn_families(b.1, 14).expect("families come to `b`");
    assert!(came.iter().all(|s| s.settlement == b.0), "they join `b`");
    sim.advance_minutes(60).expect("advances");
    furnish(&mut sim);
    let seeker = came
        .iter()
        .map(|s| s.household)
        .min()
        .expect("a newcomer of `b`");
    ripe(&mut sim, employer, 400.0);
    starve(&mut sim, &[seeker]);
    // One field 400 m from `b`'s hearth toward `a`, within the walk; the other, ripe longest, on
    // dry ground about 4 km from it the same way: farther than the work's walk (30 minutes) but
    // within what a decision's travel field reaches (1.2 times the longest walk any activity
    // allows), so its walk is known and is too long.
    let (dx, dy) = (a.0 - b.1.0, a.1 - b.1.1);
    let d = dx.hypot(dy).max(1.0);
    let near = (b.1.0 + dx / d * 400.0, b.1.1 + dy / d * 400.0);
    let far_at = spot_near(&sim, (b.1.0 + dx / d * 4000.0, b.1.1 + dy / d * 4000.0), 0);
    let fields = fields_of(&sim, employer);
    let (first, second) = (fields[0], fields[1]);
    for f in sim.land_mut_for_tests().fields.iter_mut() {
        if f.id == first {
            f.rect.x = (near.0 * 100.0) as i32 - f.rect.w / 2;
            f.rect.y = (near.1 * 100.0) as i32 - f.rect.h / 2;
        }
        if f.id == second {
            f.rect.x = (far_at.0 * 100.0) as i32 - f.rect.w / 2;
            f.rect.y = (far_at.1 * 100.0) as i32 - f.rect.h / 2;
            f.sown_day -= 2;
        }
    }
    let far = sim
        .land()
        .fields
        .iter()
        .find(|f| f.id == second)
        .map(|f| f.rect.centre_m())
        .expect("the far field");
    assert!(
        offer(&mut sim, employer).is_some_and(|o| o.0 == FieldTask::Reap),
        "short of reapers"
    );
    let (w_near, w_far) = (walk(&mut sim, seeker, near), walk(&mut sim, seeker, far));
    assert!(
        w_near.is_some_and(|w| w <= 30.0),
        "the near field within the walk: {w_near:?}"
    );
    assert!(
        w_far.is_some_and(|w| w > 30.0),
        "the work reachable, beyond the walk: {w_far:?}"
    );
    assert!(
        pool(&mut sim, seeker).contains(&(employer, true)),
        "listed, of another settlement, by its near field"
    );
    let (seen, found) = share_of(&mut sim, seeker, employer);
    assert!(seen > 0, "weighed in some of its keyed searches");
    assert_eq!(found, 0, "passed over whenever weighed ({seen} times)");
}
