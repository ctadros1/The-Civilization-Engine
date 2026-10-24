//! Policy templates (`kind = "policy"`, ADR-0013 §3): what a law of this kind does, the issues
//! whose presence makes proposing it a move, and the levels a sponsor may put forward. Whether
//! anyone proposes it, and whether it passes, is decided by people at run time; a template
//! carries no weight toward being chosen.

use civ_agents::polity::{
    Body, IssueKind, Membership, PassRule, PolicyDef, PolicyKind, RuleDef, RuleTemplate, WALL_KINDS,
};
use civ_agents::seats::{SeatMethod, Seats};
use serde::Deserialize;

/// The `kind` value of a policy template.
pub const KIND: &str = "policy";
/// The id segment: `pack:policy/name`.
pub const ID_KIND: &str = "policy";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PolicyFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub description: String,
    /// What the kernel does under it: `common_store`, `keep_store`, `against_taking`,
    /// `keep_watch`, `curfew`, `amend_body` or `repeal` (content API 49), `claim_place` (59),
    /// `agreement` (62), `build_crossing` (67), `close_source` or `dig_well` (M6a), or
    /// `build_bank` (88) or `keep_bank` (90), or `building_rule` (93), or `build_way` (138), or
    /// `plan` (141), or `build_line` (171).
    pub does: String,
    /// The issues it answers: `food_short`, `store_unkept`, `takings`, `overruled`, `petition`,
    /// `founding` (content API 49), `outsiders` (59), `claimed_from_us`, `terms_sought` (61) or
    /// `fords` (67), `sickness`, `source_suspected` or `water_far` (M6a), `flooded` (88),
    /// `bank_worn` (90), `burned` (93) or `carts_wait` (138), `no_ground`, `way_built_over` or
    /// `new_ground` (141).
    pub answers: Vec<String>,
    /// For a common store: the shares of threshed grain a sponsor may propose for the levy.
    #[serde(default)]
    pub levy_shares: Vec<f64>,
    /// For a common store: the most food one ask brings a household from it, days of its need.
    #[serde(default)]
    pub relief_days: f64,
    /// For a law against taking (content API 36): the bundles a sponsor may propose, lightest
    /// first.
    #[serde(default)]
    pub bundles: Vec<BundleFile>,
    /// For a curfew (content API 38): the hours a sponsor may propose, each `[from, to]` hours of
    /// the day, past midnight when `to` is the smaller.
    #[serde(default)]
    pub hours: Vec<[u8; 2]>,
    /// For an amendment of the custom (content API 40): who may belong (`adults`, `elders`,
    /// `landholders`; content API 118, M7c slice BZ: `organized`, the members of organizations the
    /// polity recognizes, and `privileged`, those of organizations it grants a privilege), the
    /// shares of members who must come, and the rules (`more_for`,
    /// `two_thirds`) a sponsor may combine into the body they propose.
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub quorum_shares: Vec<f64>,
    #[serde(default)]
    pub pass: Vec<String>,
    /// For an amendment to a council (content API 106, M7a slice BO, ADR-0024 §1): the seat
    /// counts, the ways seats are filled (`election`, `lot`) and the terms in days a sponsor may
    /// combine; `members` then names the councils' electorates. Left out, the template's bodies
    /// are bodies of all their members.
    #[serde(default)]
    pub seats: Vec<u8>,
    #[serde(default)]
    pub seat_methods: Vec<String>,
    #[serde(default)]
    pub seat_terms: Vec<u16>,
    /// How a council's seats may be held among a polity's settlements (content API 108, M7a
    /// slice BQ): `at_large` (the whole polity elects them all) and `by_settlement` (each
    /// settlement elects its share). Left out, `at_large` alone.
    #[serde(default)]
    pub seat_districts: Vec<String>,
    /// The question people take positions on (content API 41, M4c slice AG): "whether to keep a
    /// common store". Left out, nobody holds a position on it.
    #[serde(default)]
    pub question: Option<String>,
    /// How a law of it bears on each value (content API 43, M4c slice AG): value id to −1
    /// (against it) .. 1 (for it). Left out, it bears on none.
    #[serde(default)]
    pub bears: std::collections::BTreeMap<String, f64>,
    /// For a rule for building (content API 93, M6b slice BH): what it requires.
    #[serde(default)]
    pub rule: Option<RuleFile>,
    /// For an office (`keep_store`, `keep_watch`): a term and how it is filled (content API 105,
    /// M7a slice BN, ADR-0024 §2). Absent, the office is held until its holder dies or leaves,
    /// and the body names its holder.
    #[serde(default)]
    pub tenure: Option<TenureFile>,
    /// For a forum (`hear_cases`, content API 110, M7b slice BR, ADR-0025 §1): who sits and what
    /// it hears.
    #[serde(default)]
    pub forum: Option<ForumFile>,
    /// For a limit (`limit`, content API 112, M7b slice BT): what it forbids.
    #[serde(default)]
    pub limit: Option<LimitFile>,
    /// For a duty (`duty`, content API 114, M7c slice BV): what it is laid on. Its rates are its
    /// `levy_shares`.
    #[serde(default)]
    pub duty: Option<DutyFile>,
    /// For a privilege (`privilege`, content API 116, M7c slice BX): what it grants. Its
    /// `bundles` are what a finding of a breach imposes.
    #[serde(default)]
    pub privilege: Option<PrivilegeFile>,
    /// For debts heard (`debts_heard`, content API 122, M7d slice CB): what a finding does.
    #[serde(default)]
    pub debts: Option<DebtsFile>,
    /// For lending from the store (`lend_from_store`, content API 123, M7d slice CC): its terms.
    #[serde(default)]
    pub lending: Option<LendingFile>,
    /// For a mint (`mint`, content API 127, M7d slice CE): what it strikes.
    #[serde(default)]
    pub mint: Option<MintFile>,
    /// For pay for common work (`pay_work`, content API 128, M7d slice CE3a): what an hour is
    /// paid.
    #[serde(default)]
    pub pay: Option<PayFile>,
    /// For a common store (`common_store`, content API 129, M7d slice CE3b): a money its levy
    /// may be paid in.
    #[serde(default)]
    pub medium: Option<MediumFile>,
    /// For a plan (`plan`, content API 141, M8c slice CT): its grid.
    #[serde(default)]
    pub plan: Option<PlanFile>,
    /// For a building raised in common (`raise_building`, content API 153, M8d slice DC): what
    /// is raised.
    #[serde(default)]
    pub raise: Option<RaiseFile>,
}

/// A building raised in common in a policy file (content API 153, M8d slice DC).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RaiseFile {
    /// The building program raised, by content id: a store raised by law shelters the common
    /// store once it stands.
    pub program: String,
}

/// A plan's grid in a policy file (content API 141, M8c slice CT; ADR-0030 §2): an orthogonal
/// subdivision, metres.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlanFile {
    /// A block's sides: along its long side, and across.
    pub block_m: [f64; 2],
    /// A parcel's frontage on the block's long side, and its depth (at most half the block's
    /// width).
    pub parcel_m: [f64; 2],
    /// Every street's width.
    pub street_m: f64,
    /// The extents a sponsor chooses among, blocks along and across, smallest first: the least
    /// whose parcels would house the households its members remember forming or arriving, twice
    /// over, else the largest.
    pub extents: Vec<[u8; 2]>,
    /// Whether the block at the origin is kept as a square.
    #[serde(default)]
    pub square: bool,
    /// Hours staking out one parcel takes, as each household reckons its share.
    pub stake_hours: f64,
    /// The technique a sponsor must know to put it forward.
    #[serde(default)]
    pub needs: Option<String>,
    /// How its keeper gives its parcels (content API 142, M8c slice CU): `need` (the default),
    /// `lot`, `offer` or `standing`.
    #[serde(default)]
    pub allot: Option<String>,
    /// For `allot = "offer"`: the good and amount a household pays the store for a parcel, once.
    #[serde(default)]
    pub fee: Option<PlanFeeFile>,
    /// For `allot = "offer"`: the food and amount a household owes the store each year it holds
    /// a parcel, as ground rent (content API 143).
    #[serde(default)]
    pub rent: Option<PlanFeeFile>,
    /// How many phases its parcels open in, outward from where it is laid (content API 144):
    /// 1 to 8, 1 when unnamed.
    #[serde(default)]
    pub phases: Option<u8>,
    /// The share of the parcels of the phases open that must be given before the next opens,
    /// above 0 and at most 1 (content API 144): three quarters when unnamed.
    #[serde(default)]
    pub open_at: Option<f64>,
    /// What it does with the fields under the ground it reserves (content API 145): `around`
    /// (the default), `taken` or `paid`.
    #[serde(default)]
    pub fields: Option<String>,
    /// For `fields = "paid"`: the good and amount a hectare the store offers a field's holder.
    #[serde(default)]
    pub pay: Option<PlanPayFile>,
    /// Its districts (content API 149, M8c slice CW): the uses each phase's parcels allow; a
    /// phase named in none allows every use.
    #[serde(default)]
    pub districts: Vec<PlanDistrictFile>,
}

/// A plan's district (content API 149): the parcels of one phase, from 1, and the uses allowed
/// there, of `dwelling`, `store` and `work`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlanDistrictFile {
    pub phase: u8,
    pub uses: Vec<String>,
}

/// What a plan's store offers a hectare for a field under its ground (content API 145).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlanPayFile {
    /// The good, by content id.
    pub good: String,
    /// Units of it a hectare.
    pub per_ha: f64,
}

/// What a parcel given for a fee costs, in a plan's `[plan.fee]` table (content API 142).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlanFeeFile {
    /// The good, by content id.
    pub good: String,
    /// Units of it, paid once.
    pub amount: f64,
}

/// A money a common store's levy may be paid in, in a policy file (content API 129, M7d slice
/// CE3b).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MediumFile {
    /// The money, by content id.
    pub good: String,
    /// Units of it that pay for a kilogram of the grain owed.
    pub per_kg: f64,
}

/// What an hour given to the common works is paid in a policy file (content API 128, M7d slice
/// CE3a).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PayFile {
    /// The good paid, by content id: a money.
    pub good: String,
    /// Units of it an hour.
    pub per_hour: f64,
}

/// A mint's terms in a policy file (content API 127, M7d slice CE): the recipe it strikes by
/// (its first input the metal, its first output the coin good), and each piece of its issues.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MintFile {
    /// The recipe, by content id.
    pub recipe: String,
    /// Grams a piece weighs.
    pub mass_g: f64,
    /// The share of a piece that is the metal.
    pub fineness: f64,
    /// The share of the pieces struck from what a household brings that the polity keeps.
    pub fee: f64,
}

/// A law of lending from the store's terms in a policy file (content API 123, M7d slice CC).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LendingFile {
    /// The increment owed on what is lent, a share of it.
    pub increment: f64,
    /// The most of the food the store holds it lends at once, a share.
    pub share: f64,
}

/// A law of debts heard's terms in a policy file (content API 122, M7d slice CB).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DebtsFile {
    /// `time` (the loan is open again, due later) or `imposed` (what is owed is imposed as a
    /// finding's obligation).
    pub remedy: String,
    /// The share of what is owed households expect a finding to move, for their forecasts.
    pub share: f64,
    /// For `time`: the days after the finding the loan falls due again.
    #[serde(default)]
    pub extend_days: u32,
}

/// A privilege's terms in a policy file (content API 116, M7c slice BX).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PrivilegeFile {
    /// `licence` (the sole right to offer the goods at doors in the settlement) or `staple` (the
    /// sole right to sell there such goods fetched from another settlement).
    pub grant: String,
    /// The purpose of the goods: `food`, `fuel`, `material`, `tool` or `store`.
    pub purpose: String,
}

/// A duty's terms in a policy file (content API 114, M7c slice BV).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DutyFile {
    /// The purpose of the goods it is laid on: `food`, `fuel`, `material`, `tool` or `store`.
    pub purpose: String,
    /// Whether households that offer such goods at home are spared it.
    #[serde(default)]
    pub exempt_sellers: bool,
    /// Whether the law names one who collects it (content API 117, M7c slice BY).
    #[serde(default)]
    pub collector: bool,
}

/// A forum's terms in a policy file (content API 110).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ForumFile {
    /// `holder` (one the body names), `panel` (drawn by lot for each sitting) or `elders`.
    pub seat: String,
    /// For a panel: how many are drawn (3 to 15).
    #[serde(default)]
    pub panel: u8,
    /// The kinds of law whose cases it hears (`does` names).
    pub hears: Vec<String>,
    /// For a panel or the elders: the share of those who sit who must come (all who sit are
    /// called) and the rule, as for a body.
    #[serde(default = "half")]
    pub quorum_share: f64,
    #[serde(default = "more_for")]
    pub pass: String,
    /// How far its sitters follow remembered findings (content API 111, M7b slice BS):
    /// `informative` (the default), `persuasive` or `binding`.
    #[serde(default = "informative")]
    pub authority: String,
    /// Whether it may hear challenges of laws under limits in force (content API 112, M7b slice
    /// BT).
    #[serde(default)]
    pub review: bool,
}

/// A limit's terms in a policy file (content API 112, M7b slice BT).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LimitFile {
    /// `daylight_work` or `levy_share`.
    pub predicate: String,
    /// For daylight work: minutes of the longest day's light a curfew may cover.
    #[serde(default)]
    pub minutes: f64,
    /// For a levy: the largest share of a harvest a law may levy.
    #[serde(default)]
    pub share: f64,
    /// The rule its change is decided by (`more_for` or `two_thirds`).
    pub change: String,
}

fn informative() -> String {
    "informative".to_owned()
}

fn half() -> f64 {
    0.5
}

fn more_for() -> String {
    "more_for".to_owned()
}

/// What a rule for building requires (M6b slice BH; ADR-0022 §12).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuleFile {
    /// `roofs_unburnt`: a new roof within `within_m` of another building must not burn;
    /// `walls_of_kinds`: new walls must be of one of `walls` (content API 94); `least_gap`: a
    /// new building must stand `gap_m` from every other (94); `most_storeys`: a new building
    /// whose walls are of one of `walls` rises at most `storeys` (152); `tied_walls`: new mass
    /// walls above a storey are tied at each floor (152).
    pub template: String,
    /// For `roofs_unburnt`: how near another building a roof is bound, metres.
    #[serde(default)]
    pub within_m: f64,
    /// For `walls_of_kinds`: the wall kinds allowed, of `wattle`, `rubble`, `mud_brick` and
    /// `fired_brick`.
    #[serde(default)]
    pub walls: Vec<String>,
    /// For `least_gap`: the least gap between a new building's plot and every other's, metres.
    #[serde(default)]
    pub gap_m: f64,
    /// For `fire_from_homes` and `midden_from_wells` (content API 148): how far a new home
    /// where fire is worked keeps from others' homes, or a new home and its midden from every
    /// open well, metres.
    #[serde(default)]
    pub from_m: f64,
    /// For `most_storeys` (content API 152): the most storeys a new building whose walls are of
    /// one of `walls` may rise.
    #[serde(default)]
    pub storeys: u8,
    /// The technique a sponsor must know to put it forward (research 11-09 §1.4).
    #[serde(default)]
    pub needs: Option<String>,
}

impl RuleFile {
    /// The wall kinds `walls` names, a bit for each.
    fn wall_kinds(&self) -> u8 {
        self.walls.iter().fold(0u8, |m, w| {
            WALL_KINDS
                .iter()
                .position(|k| k == w)
                .map_or(m, |k| m | (1 << k))
        })
    }

    /// Its template, if the name is one.
    fn template(&self) -> Option<RuleTemplate> {
        match self.template.as_str() {
            "roofs_unburnt" => Some(RuleTemplate::RoofsUnburnt {
                within_m: self.within_m as f32,
            }),
            "walls_of_kinds" => Some(RuleTemplate::WallsOfKinds {
                kinds: self.wall_kinds(),
            }),
            "most_storeys" => Some(RuleTemplate::MostStoreys {
                storeys: self.storeys,
                kinds: self.wall_kinds(),
            }),
            "tied_walls" => Some(RuleTemplate::TiedWalls),
            "least_gap" => Some(RuleTemplate::LeastGap {
                gap_m: self.gap_m as f32,
            }),
            "fire_from_homes" => Some(RuleTemplate::FireFromHomes {
                from_m: self.from_m as f32,
            }),
            "midden_from_wells" => Some(RuleTemplate::MiddenFromWells {
                from_m: self.from_m as f32,
            }),
            _ => None,
        }
    }
}
/// One sanction bundle of a law against taking: what it adds to giving back what was taken, in
/// days of the taker's household's food.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleFile {
    pub compensation_days: f64,
    pub fine_days: f64,
    /// The one found to have taken is sent from the valley.
    #[serde(default)]
    pub exile: bool,
    /// For a plan (content API 147): the building found on its reserved ground is taken down.
    #[serde(default)]
    pub remove: bool,
    /// For a plan (content API 147): the building found may stand for this many days' food a
    /// year to the common store.
    #[serde(default)]
    pub yearly_days: f64,
}

/// An office's tenure in a policy file (content API 105).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TenureFile {
    /// Days the office is held from the day the body passes it.
    pub term_days: u32,
    /// Whether the one who held it may be named or drawn again for the next term.
    #[serde(default = "yes")]
    pub renewable: bool,
    /// `named`: the body names its holder; `lot`: a lot draws among those who put their names
    /// in.
    #[serde(default = "named")]
    pub fill: String,
}

fn yes() -> bool {
    true
}

fn named() -> String {
    "named".to_owned()
}

impl PolicyFile {
    /// A mint's terms, its recipe found by `recipe_index` (M7d slice CE).
    pub fn mint_def(
        &self,
        recipe_index: &dyn Fn(&str) -> Option<usize>,
    ) -> Option<civ_agents::polity::MintDef> {
        let m = self.mint.as_ref()?;
        Some(civ_agents::polity::MintDef {
            recipe: recipe_index(&m.recipe)?,
            mass_g: m.mass_g as f32,
            fineness: m.fineness as f32,
            fee: m.fee as f32,
        })
    }

    /// What an hour of common work is paid, its good found by `good_index` (M7d slice CE3a).
    pub fn pay_def(
        &self,
        good_index: &dyn Fn(&str) -> Option<usize>,
    ) -> Option<civ_agents::polity::PayDef> {
        let p = self.pay.as_ref()?;
        Some(civ_agents::polity::PayDef {
            good: good_index(&p.good)?,
            per_hour: p.per_hour as f32,
        })
    }

    /// The fee a plan's parcel is given for, its good found by `good_index` (M8c slice CU).
    pub fn plan_fee(&self, good_index: &dyn Fn(&str) -> Option<usize>) -> Option<(usize, f32)> {
        let f = self.plan.as_ref()?.fee.as_ref()?;
        Some((good_index(&f.good)?, f.amount as f32))
    }

    /// The yearly ground rent a plan's parcel is given for, its good found by `good_index` (M8c
    /// slice CU, third part).
    pub fn plan_rent(&self, good_index: &dyn Fn(&str) -> Option<usize>) -> Option<(usize, f32)> {
        let f = self.plan.as_ref()?.rent.as_ref()?;
        Some((good_index(&f.good)?, f.amount as f32))
    }

    /// What a plan's store offers a hectare for a field under its ground, its good found by
    /// `good_index` (M8c slice CU, fifth part).
    pub fn plan_pay(&self, good_index: &dyn Fn(&str) -> Option<usize>) -> Option<(usize, f32)> {
        let f = self.plan.as_ref()?.pay.as_ref()?;
        Some((good_index(&f.good)?, f.per_ha as f32))
    }

    /// The money a common store's levy may be paid in, its good found by `good_index` (M7d slice
    /// CE3b).
    pub fn medium_def(
        &self,
        good_index: &dyn Fn(&str) -> Option<usize>,
    ) -> Option<civ_agents::polity::MediumDef> {
        let m = self.medium.as_ref()?;
        Some(civ_agents::polity::MediumDef {
            good: good_index(&m.good)?,
            per_kg: m.per_kg as f32,
        })
    }

    /// The compiled template (call after [`PolicyFile::problems`] found none, and with every
    /// value `bears` names defined: `value_index` finds it).
    pub fn def(
        &self,
        value_index: &dyn Fn(&str) -> Option<usize>,
        technique_index: &dyn Fn(&str) -> Option<usize>,
    ) -> PolicyDef {
        let mut bears: Vec<(u16, f32)> = self
            .bears
            .iter()
            .filter_map(|(id, &b)| Some((u16::try_from(value_index(id)?).ok()?, b as f32)))
            .collect();
        bears.sort_by_key(|b| b.0);
        let mut levy_shares = self.levy_shares.clone();
        levy_shares.sort_by(f64::total_cmp);
        PolicyDef {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            kind: PolicyKind::from_name(&self.does).unwrap_or(PolicyKind::CommonStore),
            answers: self
                .answers
                .iter()
                .filter_map(|a| IssueKind::from_name(a))
                .collect(),
            levy_shares,
            relief_days: self.relief_days,
            bundles: self
                .bundles
                .iter()
                .map(|b| civ_agents::polity::Sanction {
                    compensation_days: b.compensation_days as f32,
                    fine_days: b.fine_days as f32,
                    exile: b.exile,
                    remove: b.remove,
                    yearly_days: b.yearly_days as f32,
                })
                .collect(),
            hours: self.hours.iter().map(|h| (h[0], h[1])).collect(),
            bodies: self.bodies(),
            question: self.question.clone(),
            bears,
            // The program is found by content id where the policies are compiled.
            raise: None,
            rule: self.rule.as_ref().and_then(|r| {
                Some(RuleDef {
                    template: r.template()?,
                    needs: r.needs.as_deref().and_then(technique_index),
                })
            }),
            tenure: self.tenure.as_ref().map(|t| civ_agents::polity::Tenure {
                term_days: t.term_days,
                renewable: t.renewable,
                by_lot: t.fill == "lot",
            }),
            forum: self.forum.as_ref().and_then(|f| {
                Some(civ_agents::forum::ForumDef {
                    seat: civ_agents::forum::ForumSeat::from_name(&f.seat)?,
                    panel: f.panel,
                    hears: f
                        .hears
                        .iter()
                        .filter_map(|h| PolicyKind::from_name(h))
                        .collect(),
                    rule: civ_agents::polity::Body {
                        members: civ_agents::polity::Membership::Adults,
                        quorum_share: f.quorum_share as f32,
                        pass: civ_agents::polity::PassRule::from_name(&f.pass)?,
                        seats: None,
                    },
                    authority: civ_agents::forum::Authority::from_name(&f.authority)?,
                    review: f.review,
                })
            }),
            // The recipe and the goods are resolved apart ([`PolicyFile::mint_def`],
            // [`PolicyFile::pay_def`], [`PolicyFile::medium_def`]).
            mint: None,
            pay: None,
            medium: None,
            plan: self.plan.as_ref().map(|g| civ_agents::polity::PlanDef {
                grid: civ_land::plans::Grid {
                    block_m: (g.block_m[0] as f32, g.block_m[1] as f32),
                    parcel_m: (g.parcel_m[0] as f32, g.parcel_m[1] as f32),
                    street_m: g.street_m as f32,
                    blocks: g.extents.first().map_or((1, 1), |e| (e[0], e[1])),
                    square: g.square,
                },
                extents: g.extents.iter().map(|e| (e[0], e[1])).collect(),
                stake_h: g.stake_hours as f32,
                needs: g.needs.as_deref().and_then(technique_index),
                allot: g
                    .allot
                    .as_deref()
                    .and_then(civ_land::AllotRule::from_name)
                    .unwrap_or(civ_land::AllotRule::Need),
                // The fee's and rent's goods are resolved apart ([`PolicyFile::plan_fee`]).
                fee: None,
                rent: None,
                phases: g.phases.unwrap_or(1).max(1),
                open_at: g.open_at.unwrap_or(0.75) as f32,
                fields: g
                    .fields
                    .as_deref()
                    .and_then(civ_agents::polity::FieldClause::from_name)
                    .unwrap_or_default(),
                // Resolved apart ([`PolicyFile::plan_pay`]).
                pay: None,
                districts: g
                    .districts
                    .iter()
                    .map(|d| {
                        let uses = d
                            .uses
                            .iter()
                            .filter_map(|u| civ_land::PlotUse::from_name(u))
                            .fold(0u8, |m, u| m | civ_agents::polity::use_bit(u));
                        (d.phase.saturating_sub(1), uses)
                    })
                    .collect(),
            }),
            lending: self
                .lending
                .as_ref()
                .map(|l| civ_agents::polity::LendingDef {
                    increment: l.increment as f32,
                    share: l.share as f32,
                }),
            debts: self.debts.as_ref().and_then(|d| {
                Some(civ_agents::polity::DebtsDef {
                    remedy: civ_agents::polity::DebtRemedy::from_name(&d.remedy)?,
                    share: d.share as f32,
                    extend_days: d.extend_days,
                })
            }),
            privilege: self.privilege.as_ref().and_then(|g| {
                Some(civ_agents::polity::PrivilegeDef {
                    grant: civ_agents::polity::Grant::from_name(&g.grant)?,
                    purpose: civ_agents::params::GoodUse::from_name(&g.purpose)?,
                })
            }),
            duty: self.duty.as_ref().and_then(|d| {
                Some(civ_agents::polity::DutyDef {
                    purpose: civ_agents::params::GoodUse::from_name(&d.purpose)?,
                    exempt_sellers: d.exempt_sellers,
                    collector: d.collector,
                })
            }),
            limit: self.limit.as_ref().and_then(|l| {
                let predicate = civ_agents::forum::Predicate::from_name(&l.predicate)?;
                Some(civ_agents::forum::LimitDef {
                    predicate,
                    bound: match predicate {
                        civ_agents::forum::Predicate::DaylightWork => l.minutes,
                        civ_agents::forum::Predicate::LevyShare => l.share,
                    },
                    change: civ_agents::polity::PassRule::from_name(&l.change)?,
                })
            }),
        }
    }

    /// For an amendment: every body its levels combine, in the order written (memberships, then
    /// quorum shares, then rules).
    fn bodies(&self) -> Vec<Body> {
        if PolicyKind::from_name(&self.does) != Some(PolicyKind::AmendBody) {
            return Vec::new();
        }
        // A council's seats, each way they may be combined (M7a slice BO): none for a body
        // of all its members.
        let mut councils: Vec<Option<(u8, SeatMethod, u16, bool)>> = Vec::new();
        let districts: Vec<bool> = if self.seat_districts.is_empty() {
            vec![false]
        } else {
            self.seat_districts
                .iter()
                .filter_map(|d| match d.as_str() {
                    "at_large" => Some(false),
                    "by_settlement" => Some(true),
                    _ => None,
                })
                .collect()
        };
        for &n in &self.seats {
            for method in self
                .seat_methods
                .iter()
                .filter_map(|m| SeatMethod::from_name(m))
            {
                for &t in &self.seat_terms {
                    for &d in &districts {
                        councils.push(Some((n, method, t, d)));
                    }
                }
            }
        }
        if councils.is_empty() {
            councils.push(None);
        }
        let mut out = Vec::new();
        for m in self.members.iter().filter_map(|m| Membership::from_name(m)) {
            for &q in &self.quorum_shares {
                for r in self.pass.iter().filter_map(|r| PassRule::from_name(r)) {
                    for c in &councils {
                        let seats = c.map(|(n, method, term_days, by_settlement)| Seats {
                            n,
                            electorate: m,
                            method,
                            term_days,
                            by_settlement,
                        });
                        out.push(Body {
                            members: if seats.is_some() {
                                Membership::Seats
                            } else {
                                m
                            },
                            quorum_share: q as f32,
                            pass: r,
                            seats,
                        });
                    }
                }
            }
        }
        out
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        for (id, &b) in &self.bears {
            if !(b.is_finite() && (-1.0..=1.0).contains(&b)) {
                p.push(format!(
                    "`bears` gives `{id}` {b}; it must be between -1 and 1"
                ));
            }
        }
        if PolicyKind::from_name(&self.does).is_none() {
            let all: Vec<&str> = PolicyKind::ALL.iter().map(|k| k.name()).collect();
            p.push(format!("`does` must be one of `{}`", all.join("`, `")));
        }
        if self.answers.is_empty() {
            p.push("`answers` must name at least one issue".to_owned());
        }
        for a in &self.answers {
            if IssueKind::from_name(a).is_none() {
                let all: Vec<&str> = IssueKind::ALL.iter().map(|k| k.name()).collect();
                p.push(format!(
                    "`answers` names `{a}`; issues are `{}`",
                    all.join("`, `")
                ));
            }
        }
        // A forum's terms (M7b slice BR): only for `hear_cases`, which must have them.
        match (&self.forum, PolicyKind::from_name(&self.does)) {
            (Some(f), Some(PolicyKind::HearCases)) => {
                if civ_agents::forum::ForumSeat::from_name(&f.seat).is_none() {
                    p.push(format!(
                        "`forum.seat` must be `holder`, `panel` or `elders` (got `{}`)",
                        f.seat
                    ));
                }
                if f.seat == "panel" && !(3..=15).contains(&f.panel) {
                    p.push(format!("`forum.panel` must be 3 to 15 (got {})", f.panel));
                }
                if f.hears.is_empty() {
                    p.push("`forum.hears` must name at least one kind of law".to_owned());
                }
                for h in &f.hears {
                    if !matches!(
                        PolicyKind::from_name(h),
                        Some(
                            PolicyKind::AgainstTaking
                                | PolicyKind::AgainstRaiding
                                | PolicyKind::Privilege
                                | PolicyKind::Plan
                        )
                    ) {
                        p.push(format!(
                            "`forum.hears` names `{h}`; cases are brought under `against_taking`, `against_raiding`, `privilege` and `plan`"
                        ));
                    }
                }
                if !(0.0..=1.0).contains(&f.quorum_share) {
                    p.push("`forum.quorum_share` must be 0 to 1".to_owned());
                }
                if civ_agents::polity::PassRule::from_name(&f.pass).is_none() {
                    p.push(format!("`forum.pass` names `{}`", f.pass));
                }
                if civ_agents::forum::Authority::from_name(&f.authority).is_none() {
                    p.push(format!(
                        "`forum.authority` must be `informative`, `persuasive` or `binding` (got \
                         `{}`)",
                        f.authority
                    ));
                }
            }
            (None, Some(PolicyKind::HearCases)) => {
                p.push("a `hear_cases` template needs a `forum` table".to_owned());
            }
            (Some(_), _) => p.push("`forum` is for `does = \"hear_cases\"`".to_owned()),
            (None, _) => {}
        }
        // A privilege's terms (M7c slice BX): only for `privilege`, which must have them.
        match (&self.privilege, PolicyKind::from_name(&self.does)) {
            (Some(g), Some(PolicyKind::Privilege)) => {
                if civ_agents::polity::Grant::from_name(&g.grant).is_none() {
                    p.push(format!(
                        "`privilege.grant` must be `licence` or `staple` (got `{}`)",
                        g.grant
                    ));
                }
                if civ_agents::params::GoodUse::from_name(&g.purpose).is_none() {
                    p.push(format!(
                        "`privilege.purpose` must be `food`, `fuel`, `material`, `tool` or \
                         `store` (got `{}`)",
                        g.purpose
                    ));
                }
            }
            (None, Some(PolicyKind::Privilege)) => {
                p.push("a `privilege` template needs a `privilege` table".to_owned());
            }
            (Some(_), _) => p.push("`privilege` is for `does = \"privilege\"`".to_owned()),
            (None, _) => {}
        }
        // A law of lending from the store's terms (M7d slice CC): only for `lend_from_store`,
        // which must have them.
        match (&self.lending, PolicyKind::from_name(&self.does)) {
            (Some(l), Some(PolicyKind::LendFromStore)) => {
                for (name, v, hi) in [
                    ("lending.increment", l.increment, 2.0),
                    ("lending.share", l.share, 1.0),
                ] {
                    if !(v.is_finite() && (0.0..=hi).contains(&v)) {
                        p.push(format!("`{name}` must be between 0 and {hi} (got {v})"));
                    }
                }
            }
            (None, Some(PolicyKind::LendFromStore)) => {
                p.push("a `lend_from_store` template needs a `lending` table".to_owned());
            }
            (Some(_), _) => p.push("`lending` is for `does = \"lend_from_store\"`".to_owned()),
            (None, _) => {}
        }
        // A mint's terms (M7d slice CE): only for `mint`, which must have them.
        match (&self.mint, PolicyKind::from_name(&self.does)) {
            (Some(m), Some(PolicyKind::Mint)) => {
                if !(m.mass_g.is_finite() && m.mass_g > 0.0 && m.mass_g <= 1000.0) {
                    p.push(format!(
                        "`mint.mass_g` must be above 0 and at most 1000 (got {})",
                        m.mass_g
                    ));
                }
                if !(m.fineness.is_finite() && m.fineness > 0.0 && m.fineness <= 1.0) {
                    p.push(format!(
                        "`mint.fineness` must be above 0 and at most 1 (got {})",
                        m.fineness
                    ));
                }
                if !(m.fee.is_finite() && (0.0..1.0).contains(&m.fee)) {
                    p.push(format!(
                        "`mint.fee` must be at least 0 and below 1 (got {})",
                        m.fee
                    ));
                }
            }
            (None, Some(PolicyKind::Mint)) => {
                p.push("a `mint` template needs a `mint` table".to_owned());
            }
            (Some(_), _) => p.push("`mint` is for `does = \"mint\"`".to_owned()),
            (None, _) => {}
        }
        // A plan's grid (M8c slice CT): only for `plan`, which must have one.
        match (&self.plan, PolicyKind::from_name(&self.does)) {
            (Some(g), Some(PolicyKind::Plan)) => {
                let metres = |m: f64, lo: f64, hi: f64| m.is_finite() && (lo..=hi).contains(&m);
                if !(metres(g.block_m[0], 4.0, 400.0) && metres(g.block_m[1], 4.0, 400.0)) {
                    p.push(format!(
                        "`plan.block_m` must be 4 to 400 m each way (got {:?})",
                        g.block_m
                    ));
                }
                if !(metres(g.parcel_m[0], 2.0, g.block_m[0])
                    && metres(g.parcel_m[1], 2.0, g.block_m[1] / 2.0))
                {
                    p.push(format!(
                        "`plan.parcel_m` must be 2 m or more, its frontage no more than a block's \
                         length and its depth no more than half a block's width (got {:?})",
                        g.parcel_m
                    ));
                }
                if !metres(g.street_m, 1.0, 60.0) {
                    p.push(format!(
                        "`plan.street_m` must be 1 to 60 m (got {})",
                        g.street_m
                    ));
                }
                if !(1..=8).contains(&g.extents.len())
                    || g.extents
                        .iter()
                        .any(|e| !(1..=16).contains(&e[0]) || !(1..=16).contains(&e[1]))
                {
                    p.push(
                        "`plan.extents` must hold 1 to 8 extents of 1 to 16 blocks each way"
                            .to_owned(),
                    );
                }
                if !metres(g.stake_hours, 0.0, 100.0) {
                    p.push(format!(
                        "`plan.stake_hours` must be 0 to 100 (got {})",
                        g.stake_hours
                    ));
                }
                let allot = g.allot.as_deref().map(civ_land::AllotRule::from_name);
                if allot == Some(None) {
                    p.push(format!(
                        "`plan.allot` must be one of need, lot, offer or standing (got {:?})",
                        g.allot.as_deref().unwrap_or_default()
                    ));
                }
                let offer = allot == Some(Some(civ_land::AllotRule::Offer));
                if offer && g.fee.is_none() && g.rent.is_none() {
                    p.push(
                        "`plan.allot = \"offer\"` needs a `plan.fee` or `plan.rent` table"
                            .to_owned(),
                    );
                }
                if let Some(n) = g.phases
                    && !(1..=8).contains(&n)
                {
                    p.push(format!("`plan.phases` must be 1 to 8 (got {n})"));
                }
                // Districts (content API 149): each a phase of the plan's, named once, allowing
                // at least one use, each a use the kernel knows.
                let phases = g.phases.unwrap_or(1).max(1);
                for (i, d) in g.districts.iter().enumerate() {
                    if !(1..=phases).contains(&d.phase) {
                        p.push(format!(
                            "a `plan.districts` phase must be 1 to the plan's phases, {phases} (got \
                             {})",
                            d.phase
                        ));
                    }
                    if g.districts[..i].iter().any(|e| e.phase == d.phase) {
                        p.push(format!("`plan.districts` names phase {} twice", d.phase));
                    }
                    if d.uses.is_empty() {
                        p.push("a `plan.districts` entry must allow at least one use".to_owned());
                    }
                    for u in &d.uses {
                        if civ_land::PlotUse::from_name(u).is_none() {
                            p.push(format!(
                                "a `plan.districts` use must be dwelling, store or work (got {u:?})"
                            ));
                        }
                    }
                }
                if let Some(x) = g.open_at
                    && !(x > 0.0 && x <= 1.0)
                {
                    p.push(format!(
                        "`plan.open_at` must be above 0 and at most 1 (got {x})"
                    ));
                }
                let clause = g
                    .fields
                    .as_deref()
                    .map(civ_agents::polity::FieldClause::from_name);
                if clause == Some(None) {
                    p.push(format!(
                        "`plan.fields` must be one of around, taken or paid (got {:?})",
                        g.fields.as_deref().unwrap_or_default()
                    ));
                }
                let paid = clause == Some(Some(civ_agents::polity::FieldClause::Paid));
                match &g.pay {
                    None if paid => {
                        p.push("`plan.fields = \"paid\"` needs a `plan.pay` table".to_owned());
                    }
                    Some(_) if !paid => {
                        p.push("`plan.pay` is for `plan.fields = \"paid\"`".to_owned());
                    }
                    Some(f) if !(f.per_ha.is_finite() && f.per_ha > 0.0) => {
                        p.push(format!(
                            "`plan.pay.per_ha` must be more than 0 (got {})",
                            f.per_ha
                        ));
                    }
                    _ => {}
                }
                for (name, paid) in [("fee", &g.fee), ("rent", &g.rent)] {
                    match paid {
                        Some(_) if !offer => {
                            p.push(format!("`plan.{name}` is for `plan.allot = \"offer\"`"));
                        }
                        Some(f) if !(f.amount.is_finite() && f.amount > 0.0) => {
                            p.push(format!(
                                "`plan.{name}.amount` must be more than 0 (got {})",
                                f.amount
                            ));
                        }
                        _ => {}
                    }
                }
            }
            (None, Some(PolicyKind::Plan)) => {
                p.push("a `plan` template needs a `plan` table".to_owned());
            }
            (Some(_), _) => p.push("`plan` is for `does = \"plan\"`".to_owned()),
            (None, _) => {}
        }
        // A money the levy may be paid in (M7d slice CE3b): only for `common_store`.
        match (&self.medium, PolicyKind::from_name(&self.does)) {
            (Some(m), Some(PolicyKind::CommonStore)) => {
                if !(m.per_kg.is_finite() && m.per_kg > 0.0 && m.per_kg <= 10000.0) {
                    p.push(format!(
                        "`medium.per_kg` must be above 0 and at most 10000 (got {})",
                        m.per_kg
                    ));
                }
            }
            (Some(_), _) => p.push("`medium` is for `does = \"common_store\"`".to_owned()),
            (None, _) => {}
        }
        // A building raised in common (M8d slice DC): only for `raise_building`, which must
        // name one.
        match (&self.raise, PolicyKind::from_name(&self.does)) {
            (None, Some(PolicyKind::RaiseBuilding)) => {
                p.push("a `raise_building` template needs a `raise` table".to_owned());
            }
            (Some(_), Some(PolicyKind::RaiseBuilding)) | (None, _) => {}
            (Some(_), _) => p.push("`raise` is for `does = \"raise_building\"`".to_owned()),
        }
        // Pay for common work (M7d slice CE3a): only for `pay_work`, which must have it.
        match (&self.pay, PolicyKind::from_name(&self.does)) {
            (Some(pay), Some(PolicyKind::PayWork)) => {
                if !(pay.per_hour.is_finite() && pay.per_hour > 0.0 && pay.per_hour <= 1000.0) {
                    p.push(format!(
                        "`pay.per_hour` must be above 0 and at most 1000 (got {})",
                        pay.per_hour
                    ));
                }
            }
            (None, Some(PolicyKind::PayWork)) => {
                p.push("a `pay_work` template needs a `pay` table".to_owned());
            }
            (Some(_), _) => p.push("`pay` is for `does = \"pay_work\"`".to_owned()),
            (None, _) => {}
        }
        // A law of debts heard's terms (M7d slice CB): only for `debts_heard`, which must have
        // them.
        match (&self.debts, PolicyKind::from_name(&self.does)) {
            (Some(d), Some(PolicyKind::DebtsHeard)) => {
                let remedy = civ_agents::polity::DebtRemedy::from_name(&d.remedy);
                if remedy.is_none() {
                    p.push(format!(
                        "`debts.remedy` must be `time` or `imposed` (got `{}`)",
                        d.remedy
                    ));
                }
                if !(d.share.is_finite() && (0.0..=1.0).contains(&d.share)) {
                    p.push(format!(
                        "`debts.share` must be between 0 and 1 (got {})",
                        d.share
                    ));
                }
                if remedy == Some(civ_agents::polity::DebtRemedy::Time)
                    && !(1..=720).contains(&d.extend_days)
                {
                    p.push(format!(
                        "`debts.extend_days` must be between 1 and 720 for `time` (got {})",
                        d.extend_days
                    ));
                }
            }
            (None, Some(PolicyKind::DebtsHeard)) => {
                p.push("a `debts_heard` template needs a `debts` table".to_owned());
            }
            (Some(_), _) => p.push("`debts` is for `does = \"debts_heard\"`".to_owned()),
            (None, _) => {}
        }
        // A duty's terms (M7c slice BV): only for `duty`, which must have them.
        match (&self.duty, PolicyKind::from_name(&self.does)) {
            (Some(d), Some(PolicyKind::Duty)) => {
                if civ_agents::params::GoodUse::from_name(&d.purpose).is_none() {
                    p.push(format!(
                        "`duty.purpose` must be `food`, `fuel`, `material`, `tool` or `store` \
                         (got `{}`)",
                        d.purpose
                    ));
                }
            }
            (None, Some(PolicyKind::Duty)) => {
                p.push("a `duty` template needs a `duty` table".to_owned());
            }
            (Some(_), _) => p.push("`duty` is for `does = \"duty\"`".to_owned()),
            (None, _) => {}
        }
        // A limit's terms (M7b slice BT): only for `limit`, which must have them.
        match (&self.limit, PolicyKind::from_name(&self.does)) {
            (Some(l), Some(PolicyKind::Limit)) => {
                match civ_agents::forum::Predicate::from_name(&l.predicate) {
                    Some(civ_agents::forum::Predicate::DaylightWork) => {
                        if !(l.minutes.is_finite() && (1.0..=720.0).contains(&l.minutes)) {
                            p.push(format!(
                                "`limit.minutes` must be 1 to 720 (got {})",
                                l.minutes
                            ));
                        }
                    }
                    Some(civ_agents::forum::Predicate::LevyShare) => {
                        if !(l.share.is_finite() && l.share > 0.0 && l.share < 1.0) {
                            p.push(format!(
                                "`limit.share` must lie above 0 and below 1 (got {})",
                                l.share
                            ));
                        }
                    }
                    None => p.push(format!(
                        "`limit.predicate` must be `daylight_work` or `levy_share` (got `{}`)",
                        l.predicate
                    )),
                }
                if civ_agents::polity::PassRule::from_name(&l.change).is_none() {
                    p.push(format!("`limit.change` names `{}`", l.change));
                }
            }
            (None, Some(PolicyKind::Limit)) => {
                p.push("a `limit` template needs a `limit` table".to_owned());
            }
            (Some(_), _) => p.push("`limit` is for `does = \"limit\"`".to_owned()),
            (None, _) => {}
        }
        if let Some(t) = &self.tenure {
            if !matches!(
                PolicyKind::from_name(&self.does),
                Some(PolicyKind::KeepStore | PolicyKind::KeepWatch)
            ) {
                p.push(
                    "`tenure` is for an office: `does` must be `keep_store` or `keep_watch`"
                        .to_owned(),
                );
            }
            // A day to ten years (research 09-14 §2.1: a presiding officer for a day; 09-02
            // §5.1: life subject to removal is the absent tenure).
            if !(1..=3650).contains(&t.term_days) {
                p.push(format!(
                    "`tenure.term_days` must be 1 to 3650 (got {})",
                    t.term_days
                ));
            }
            if !matches!(t.fill.as_str(), "named" | "lot") {
                p.push(format!(
                    "`tenure.fill` must be `named` or `lot` (got `{}`); election and appointment come with seated bodies",
                    t.fill
                ));
            }
        }
        match PolicyKind::from_name(&self.does) {
            // A duty's rates (M7c slice BV): shares of the goods bought, as a store's levy.
            Some(PolicyKind::Duty) => {
                if !(1..=8).contains(&self.levy_shares.len()) {
                    p.push(format!(
                        "`levy_shares` must hold 1 to 8 rates (got {})",
                        self.levy_shares.len()
                    ));
                }
                for &s in &self.levy_shares {
                    if !(s.is_finite() && s > 0.0 && s < 1.0) {
                        p.push(format!(
                            "`levy_shares` must lie above 0 and below 1 (got {s})"
                        ));
                    }
                }
                if self.relief_days != 0.0 {
                    p.push("a `duty` policy gives no relief: leave out `relief_days`".to_owned());
                }
            }
            Some(PolicyKind::CommonStore) => {
                // Research 09-05 §2.3: a sponsor weighs three to eight levels; one is allowed.
                if !(1..=8).contains(&self.levy_shares.len()) {
                    p.push(format!(
                        "`levy_shares` must hold 1 to 8 shares (got {})",
                        self.levy_shares.len()
                    ));
                }
                for &s in &self.levy_shares {
                    if !(s.is_finite() && s > 0.0 && s < 1.0) {
                        p.push(format!(
                            "`levy_shares` must lie above 0 and below 1 (got {s})"
                        ));
                    }
                }
                let d = self.relief_days;
                if !(d.is_finite() && d > 0.0 && d <= 365.0) {
                    p.push(format!(
                        "`relief_days` must be above 0 and at most 365 (got {d})"
                    ));
                }
            }
            Some(
                k @ (PolicyKind::KeepStore
                | PolicyKind::KeepWatch
                | PolicyKind::Curfew
                | PolicyKind::AmendBody
                | PolicyKind::Repeal
                | PolicyKind::ClaimPlace
                | PolicyKind::Agreement
                | PolicyKind::BuildCrossing
                | PolicyKind::CloseSource
                | PolicyKind::DigWell
                | PolicyKind::BuildBank
                | PolicyKind::KeepBank
                | PolicyKind::BuildingRule
                | PolicyKind::SendBand
                | PolicyKind::Enclose
                | PolicyKind::Annex
                | PolicyKind::HearCases
                | PolicyKind::Limit
                | PolicyKind::Embargo
                | PolicyKind::Recognize
                | PolicyKind::DebtsHeard
                | PolicyKind::LendFromStore
                | PolicyKind::Mint
                | PolicyKind::PayWork
                | PolicyKind::BuildWay
                | PolicyKind::BuildLine
                | PolicyKind::Plan
                | PolicyKind::RaiseBuilding),
            ) if !self.levy_shares.is_empty() || self.relief_days != 0.0 => {
                p.push(format!(
                    "a `{}` policy levies nothing: leave out `levy_shares` and `relief_days`",
                    k.name()
                ));
            }
            Some(
                k
                @ (PolicyKind::AgainstTaking | PolicyKind::AgainstRaiding | PolicyKind::Privilege),
            ) => {
                if !self.levy_shares.is_empty() || self.relief_days != 0.0 {
                    p.push(format!(
                        "an `{}` policy levies nothing: leave out `levy_shares` and \
                         `relief_days`",
                        k.name()
                    ));
                }
                // Research 09-05 §2.3: a sponsor weighs a few levels; one is allowed.
                if !(1..=8).contains(&self.bundles.len()) {
                    p.push(format!(
                        "`bundles` must hold 1 to 8 bundles (got {})",
                        self.bundles.len()
                    ));
                }
                for b in &self.bundles {
                    for (what, d) in [
                        ("compensation_days", b.compensation_days),
                        ("fine_days", b.fine_days),
                    ] {
                        if !(d.is_finite() && (0.0..=365.0).contains(&d)) {
                            p.push(format!(
                                "a bundle's `{what}` must be between 0 and 365 (got {d})"
                            ));
                        }
                    }
                }
            }
            Some(
                PolicyKind::KeepStore
                | PolicyKind::KeepWatch
                | PolicyKind::Curfew
                | PolicyKind::AmendBody
                | PolicyKind::Repeal
                | PolicyKind::ClaimPlace
                | PolicyKind::Agreement
                | PolicyKind::BuildCrossing
                | PolicyKind::CloseSource
                | PolicyKind::DigWell
                | PolicyKind::BuildBank
                | PolicyKind::KeepBank
                | PolicyKind::BuildingRule
                | PolicyKind::SendBand
                | PolicyKind::Enclose
                | PolicyKind::Annex
                | PolicyKind::HearCases
                | PolicyKind::Limit
                | PolicyKind::Embargo
                | PolicyKind::Recognize
                | PolicyKind::DebtsHeard
                | PolicyKind::LendFromStore
                | PolicyKind::Mint
                | PolicyKind::PayWork
                | PolicyKind::BuildWay
                | PolicyKind::BuildLine
                | PolicyKind::Plan
                | PolicyKind::RaiseBuilding,
            )
            | None => {}
        }
        if PolicyKind::from_name(&self.does) == Some(PolicyKind::AmendBody) {
            if !(1..=Membership::ALL.len()).contains(&self.members.len()) {
                p.push(format!(
                    "`members` must hold 1 to {} memberships (got {})",
                    Membership::ALL.len(),
                    self.members.len()
                ));
            }
            for m in &self.members {
                if Membership::from_name(m).is_none() {
                    let all: Vec<&str> = Membership::ALL.iter().map(|m| m.name()).collect();
                    p.push(format!(
                        "`members` names `{m}`; memberships are `{}`",
                        all.join("`, `")
                    ));
                }
            }
            if !(1..=8).contains(&self.quorum_shares.len()) {
                p.push(format!(
                    "`quorum_shares` must hold 1 to 8 shares (got {})",
                    self.quorum_shares.len()
                ));
            }
            for &q in &self.quorum_shares {
                if !(q.is_finite() && (0.0..=1.0).contains(&q)) {
                    p.push(format!(
                        "`quorum_shares` must lie between 0 and 1 (got {q})"
                    ));
                }
            }
            if !(1..=2).contains(&self.pass.len()) {
                p.push(format!(
                    "`pass` must hold 1 or 2 rules (got {})",
                    self.pass.len()
                ));
            }
            for r in &self.pass {
                if PassRule::from_name(r).is_none() {
                    let all: Vec<&str> = PassRule::ALL.iter().map(|r| r.name()).collect();
                    p.push(format!(
                        "`pass` names `{r}`; rules are `{}`",
                        all.join("`, `")
                    ));
                }
            }
            // A council's levels come together or not at all (M7a slice BO).
            let council = [
                !self.seats.is_empty(),
                !self.seat_methods.is_empty(),
                !self.seat_terms.is_empty(),
            ];
            if council.iter().any(|&c| c) && !council.iter().all(|&c| c) {
                p.push(
                    "`seats`, `seat_methods` and `seat_terms` must be given together".to_owned(),
                );
            }
            for &n in &self.seats {
                // Research 09-05 §2.3: councils of 5, 9 and 15.
                if !(3..=31).contains(&n) {
                    p.push(format!("`seats` must each be 3 to 31 (got {n})"));
                }
            }
            for m in &self.seat_methods {
                if SeatMethod::from_name(m).is_none() {
                    p.push(format!(
                        "`seat_methods` names `{m}`; methods are `election`, `lot`"
                    ));
                }
            }
            for d in &self.seat_districts {
                if d != "at_large" && d != "by_settlement" {
                    p.push(format!(
                        "`seat_districts` names `{d}`; they are `at_large`, `by_settlement`"
                    ));
                }
            }
            if !self.seat_districts.is_empty() && !council[0] {
                p.push("`seat_districts` needs `seats`".to_owned());
            }
            for &t in &self.seat_terms {
                if !(30..=3650).contains(&t) {
                    p.push(format!(
                        "`seat_terms` must each be 30 to 3650 days (got {t})"
                    ));
                }
            }
            if self.members.iter().any(|m| m == "seats" || m == "watch") && council[0] {
                p.push(
                    "a council's electorate (`members`) is `adults`, `elders` or `landholders`"
                        .to_owned(),
                );
            }
        } else if !self.members.is_empty()
            || !self.quorum_shares.is_empty()
            || !self.pass.is_empty()
            || !self.seats.is_empty()
            || !self.seat_methods.is_empty()
            || !self.seat_terms.is_empty()
        {
            p.push(
                "only an `amend_body` policy has `members`, `quorum_shares`, `pass` and seats"
                    .to_owned(),
            );
        }
        if PolicyKind::from_name(&self.does) == Some(PolicyKind::Curfew) {
            if !(1..=8).contains(&self.hours.len()) {
                p.push(format!(
                    "`hours` must hold 1 to 8 pairs (got {})",
                    self.hours.len()
                ));
            }
            for h in &self.hours {
                if h[0] > 23 || h[1] > 23 || h[0] == h[1] {
                    p.push(format!(
                        "each of `hours` must be two different hours of the day, 0 to 23 (got \
                         {h:?})"
                    ));
                }
            }
        } else if !self.hours.is_empty() {
            p.push("only a `curfew` policy has `hours`".to_owned());
        }
        if !matches!(
            PolicyKind::from_name(&self.does),
            Some(
                PolicyKind::AgainstTaking
                    | PolicyKind::AgainstRaiding
                    | PolicyKind::Privilege
                    | PolicyKind::Plan
            )
        ) && !self.bundles.is_empty()
        {
            p.push(
                "only an `against_taking`, `against_raiding`, `privilege` or `plan` policy has \
                 `bundles`"
                    .to_owned(),
            );
        }
        // A plan names at most one bundle, what a finding of building on its reserved ground
        // imposes, and sends nobody away (content API 146, M8c slice CV).
        if PolicyKind::from_name(&self.does) == Some(PolicyKind::Plan) {
            if self.bundles.len() > 1 {
                p.push(format!(
                    "a `plan` policy names at most one bundle (got {})",
                    self.bundles.len()
                ));
            }
            for b in &self.bundles {
                if b.exile {
                    p.push("a `plan` policy's bundle exiles nobody".to_owned());
                }
                if b.remove && b.yearly_days > 0.0 {
                    p.push(
                        "a `plan` policy's bundle takes a building down or lets it stand for a \
                         yearly payment, not both"
                            .to_owned(),
                    );
                }
                for (what, d) in [
                    ("compensation_days", b.compensation_days),
                    ("fine_days", b.fine_days),
                    ("yearly_days", b.yearly_days),
                ] {
                    if !(d.is_finite() && (0.0..=365.0).contains(&d)) {
                        p.push(format!(
                            "a bundle's `{what}` must be between 0 and 365 (got {d})"
                        ));
                    }
                }
            }
        } else if self
            .bundles
            .iter()
            .any(|b| b.remove || b.yearly_days != 0.0)
        {
            p.push("only a `plan` policy's bundle has `remove` or `yearly_days`".to_owned());
        }
        match (PolicyKind::from_name(&self.does), &self.rule) {
            (Some(PolicyKind::BuildingRule), None) => {
                p.push("a `building_rule` policy needs a `[rule]`".to_owned());
            }
            (Some(PolicyKind::BuildingRule), Some(r)) => match r.template() {
                None => p.push(format!(
                    "`rule.template` is `{}`; templates are `roofs_unburnt`, `walls_of_kinds`, \
                     `least_gap`, `fire_from_homes`, `midden_from_wells`, `most_storeys` and \
                     `tied_walls`",
                    r.template
                )),
                // Research 12-03 §2.2: radiation from a burning dwelling falls from about 36
                // kW/m² at 1 m to 5 at 4 m; a rule's reach is authored within a street's width.
                Some(RuleTemplate::RoofsUnburnt { .. }) => {
                    if !(r.within_m.is_finite() && r.within_m > 0.0 && r.within_m <= 50.0) {
                        p.push(format!(
                            "`rule.within_m` must be above 0 and at most 50 (got {})",
                            r.within_m
                        ));
                    }
                }
                // A frame rises at most four storeys (ADR-0031 §1), so a limit is one to three.
                Some(RuleTemplate::MostStoreys { storeys, .. }) => {
                    if !(1..=3).contains(&storeys) {
                        p.push(format!("`rule.storeys` must be 1 to 3 (got {storeys})"));
                    }
                    p.extend(walls_named(&r.walls));
                }
                Some(RuleTemplate::TiedWalls) => {}
                Some(RuleTemplate::WallsOfKinds { .. }) => {
                    p.extend(walls_named(&r.walls));
                }
                // Research 12-03 §2.2: separation thresholds of about 2.14 and 3.14 m for its
                // configurations; a gap is authored within a street's width.
                Some(RuleTemplate::LeastGap { .. }) => {
                    if !(r.gap_m.is_finite() && r.gap_m > 0.0 && r.gap_m <= 50.0) {
                        p.push(format!(
                            "`rule.gap_m` must be above 0 and at most 50 (got {})",
                            r.gap_m
                        ));
                    }
                }
                // A home's distance from others' homes or from wells, authored within a reach
                // the kernel models: a fire's (`fire.reach_m`) or a heap's seepage
                // (`contamination.link_radius_m`), each within 50 m (content API 148).
                Some(RuleTemplate::FireFromHomes { .. } | RuleTemplate::MiddenFromWells { .. }) => {
                    if !(r.from_m.is_finite() && r.from_m > 0.0 && r.from_m <= 50.0) {
                        p.push(format!(
                            "`rule.from_m` must be above 0 and at most 50 (got {})",
                            r.from_m
                        ));
                    }
                }
            },
            (_, Some(_)) => p.push("only a `building_rule` policy has a `[rule]`".to_owned()),
            (_, None) => {}
        }
        p
    }
}

/// What is wrong with a rule's `walls`: none named, or a name that is no wall kind.
fn walls_named(walls: &[String]) -> Vec<String> {
    let mut p = Vec::new();
    if walls.is_empty() {
        p.push("`rule.walls` must name at least one wall kind".to_owned());
    }
    for w in walls {
        if !WALL_KINDS.contains(&w.as_str()) {
            p.push(format!(
                "`rule.walls` names `{w}`; wall kinds are `{}`",
                WALL_KINDS.join("`, `")
            ));
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file() -> PolicyFile {
        PolicyFile {
            kind: KIND.to_owned(),
            id: "core:policy/test".to_owned(),
            name: "Common store".to_owned(),
            description: "A test.".to_owned(),
            does: "common_store".to_owned(),
            answers: vec!["food_short".to_owned()],
            levy_shares: vec![0.2, 0.05, 0.1],
            relief_days: 5.0,
            bundles: Vec::new(),
            hours: Vec::new(),
            members: Vec::new(),
            quorum_shares: Vec::new(),
            pass: Vec::new(),
            seats: Vec::new(),
            seat_methods: Vec::new(),
            seat_terms: Vec::new(),
            seat_districts: Vec::new(),
            question: None,
            bears: Default::default(),
            rule: None,
            tenure: None,
            forum: None,
            limit: None,
            duty: None,
            privilege: None,
            debts: None,
            lending: None,
            mint: None,
            pay: None,
            medium: None,
            plan: None,
            raise: None,
        }
    }

    fn no_values(_: &str) -> Option<usize> {
        None
    }

    #[test]
    fn a_plan_names_how_its_parcels_are_given_and_a_fee_only_for_an_offer() {
        // M8c slice CU, second part (content API 142).
        let grid = |allot: Option<&str>, fee: Option<PlanFeeFile>| PlanFile {
            block_m: [80.0, 40.0],
            parcel_m: [10.0, 20.0],
            street_m: 6.0,
            extents: vec![[1, 1]],
            square: true,
            stake_hours: 2.0,
            needs: None,
            allot: allot.map(str::to_owned),
            fee,
            rent: None,
            phases: None,
            open_at: None,
            fields: None,
            pay: None,
            districts: Vec::new(),
        };
        let plan_with = |g: PlanFile| PolicyFile {
            does: "plan".to_owned(),
            answers: vec!["no_ground".to_owned()],
            levy_shares: Vec::new(),
            relief_days: 0.0,
            plan: Some(g),
            ..file()
        };
        let grain = || PlanFeeFile {
            good: "core:good/grain".to_owned(),
            amount: 2.0,
        };
        // By need, unnamed or named.
        for allot in [None, Some("need")] {
            let p = plan_with(grid(allot, None));
            assert!(p.problems().is_empty(), "{:?}", p.problems());
            let d = p.def(&no_values, &no_values);
            assert_eq!(
                d.plan.map(|g| (g.allot, g.fee)),
                Some((civ_land::AllotRule::Need, None))
            );
        }
        // For a fee, its good resolved apart.
        let p = plan_with(grid(Some("offer"), Some(grain())));
        assert!(p.problems().is_empty(), "{:?}", p.problems());
        assert_eq!(
            p.def(&no_values, &no_values).plan.map(|g| g.allot),
            Some(civ_land::AllotRule::Offer)
        );
        let grains = |id: &str| (id == "core:good/grain").then_some(7);
        assert_eq!(p.plan_fee(&grains), Some((7, 2.0)));
        // An unknown rule, an offer without a fee, a fee without an offer, a fee of nothing.
        let unknown = plan_with(grid(Some("auction"), None));
        assert!(
            unknown
                .problems()
                .iter()
                .any(|e| e.contains("`plan.allot`"))
        );
        let unpriced = plan_with(grid(Some("offer"), None));
        assert!(
            unpriced
                .problems()
                .iter()
                .any(|e| e.contains("needs a `plan.fee`"))
        );
        let priced = plan_with(grid(Some("lot"), Some(grain())));
        assert!(
            priced
                .problems()
                .iter()
                .any(|e| e.contains("`plan.fee` is for"))
        );
        let free = plan_with(grid(
            Some("offer"),
            Some(PlanFeeFile {
                amount: 0.0,
                ..grain()
            }),
        ));
        assert!(
            free.problems()
                .iter()
                .any(|e| e.contains("`plan.fee.amount`"))
        );
        // A yearly ground rent, alone or with a fee, for an offer only (content API 143).
        let rented = |allot: &str, amount: f64| {
            let mut g = grid(Some(allot), None);
            g.rent = Some(PlanFeeFile { amount, ..grain() });
            plan_with(g)
        };
        let p = rented("offer", 3.0);
        assert!(p.problems().is_empty(), "{:?}", p.problems());
        assert_eq!(p.plan_rent(&grains), Some((7, 3.0)));
        assert_eq!(p.plan_fee(&grains), None);
        assert!(
            rented("need", 3.0)
                .problems()
                .iter()
                .any(|e| e.contains("`plan.rent` is for"))
        );
        assert!(
            rented("offer", -1.0)
                .problems()
                .iter()
                .any(|e| e.contains("`plan.rent.amount`"))
        );
        // The fields under it: around, taken, or paid with a pay a hectare (content API 145).
        let claiming = |fields: Option<&str>, pay: Option<f64>| {
            let mut g = grid(None, None);
            g.fields = fields.map(str::to_owned);
            g.pay = pay.map(|per_ha| PlanPayFile {
                good: "core:good/grain".to_owned(),
                per_ha,
            });
            plan_with(g)
        };
        let around = claiming(None, None).def(&no_values, &no_values).plan;
        assert_eq!(
            around.map(|g| g.fields),
            Some(civ_agents::polity::FieldClause::Around)
        );
        for (fields, pay) in [(Some("taken"), None), (Some("paid"), Some(40.0))] {
            let p = claiming(fields, pay);
            assert!(p.problems().is_empty(), "{:?}", p.problems());
        }
        assert_eq!(
            claiming(Some("paid"), Some(40.0)).plan_pay(&grains),
            Some((7, 40.0))
        );
        for (fields, pay, problem) in [
            (Some("bought"), None, "`plan.fields`"),
            (Some("paid"), None, "needs a `plan.pay`"),
            (Some("taken"), Some(40.0), "`plan.pay` is for"),
            (Some("paid"), Some(0.0), "`plan.pay.per_ha`"),
        ] {
            assert!(
                claiming(fields, pay)
                    .problems()
                    .iter()
                    .any(|e| e.contains(problem)),
                "{fields:?} {pay:?}"
            );
        }
        // At most one bundle, what a finding of building on its ground imposes, and no exile
        // (content API 146).
        let bundle = |exile: bool| BundleFile {
            compensation_days: 2.0,
            fine_days: 1.0,
            exile,
            remove: false,
            yearly_days: 0.0,
        };
        let mut p = plan_with(grid(None, None));
        p.bundles = vec![bundle(false)];
        assert!(p.problems().is_empty(), "{:?}", p.problems());
        assert_eq!(p.def(&no_values, &no_values).bundles[0].fine_days, 1.0);
        p.bundles = vec![bundle(false), bundle(false)];
        assert!(
            p.problems()
                .iter()
                .any(|e| e.contains("at most one bundle"))
        );
        p.bundles = vec![bundle(true)];
        assert!(p.problems().iter().any(|e| e.contains("exiles nobody")));
        // Removal or a yearly payment, not both (content API 147).
        let mut b = bundle(false);
        b.remove = true;
        p.bundles = vec![b.clone()];
        assert!(p.problems().is_empty(), "{:?}", p.problems());
        assert!(p.def(&no_values, &no_values).bundles[0].remove);
        b.yearly_days = 2.0;
        p.bundles = vec![b.clone()];
        assert!(p.problems().iter().any(|e| e.contains("not both")));
        b.remove = false;
        p.bundles = vec![b.clone()];
        assert!(p.problems().is_empty(), "{:?}", p.problems());
        assert_eq!(p.def(&no_values, &no_values).bundles[0].yearly_days, 2.0);
        b.yearly_days = 400.0;
        p.bundles = vec![b];
        assert!(p.problems().iter().any(|e| e.contains("`yearly_days`")));
        // Phases, 1 to 8, opening at a share above 0 and at most 1 (content API 144).
        let phased = |phases: Option<u8>, open_at: Option<f64>| {
            let mut g = grid(None, None);
            g.phases = phases;
            g.open_at = open_at;
            plan_with(g)
        };
        // Districts by phase, each allowing the uses it names (content API 149).
        let district = |phase: u8, uses: &[&str]| PlanDistrictFile {
            phase,
            uses: uses.iter().map(|u| (*u).to_owned()).collect(),
        };
        let mut zoned = phased(Some(2), None);
        zoned.plan.as_mut().expect("a plan").districts = vec![district(2, &["work", "store"])];
        assert!(zoned.problems().is_empty(), "{:?}", zoned.problems());
        let def = zoned.def(&no_values, &no_values).plan.expect("a plan");
        assert_eq!(def.districts, vec![(1, 0b110)]);
        assert!(def.allows(0, civ_land::PlotUse::Dwelling));
        assert!(!def.allows(1, civ_land::PlotUse::Dwelling));
        assert!(def.allows(1, civ_land::PlotUse::Work));
        for (bad, problem) in [
            (vec![district(3, &["work"])], "1 to the plan's phases"),
            (vec![district(1, &[])], "at least one use"),
            (vec![district(1, &["mill"])], "dwelling, store or work"),
            (
                vec![district(1, &["work"]), district(1, &["store"])],
                "names phase 1 twice",
            ),
        ] {
            zoned.plan.as_mut().expect("a plan").districts = bad;
            assert!(
                zoned.problems().iter().any(|e| e.contains(problem)),
                "{problem}: {:?}",
                zoned.problems()
            );
        }
        let unnamed = phased(None, None).def(&no_values, &no_values).plan;
        assert_eq!(unnamed.map(|g| (g.phases, g.open_at)), Some((1, 0.75)));
        let p = phased(Some(3), Some(0.5));
        assert!(p.problems().is_empty(), "{:?}", p.problems());
        let named = p.def(&no_values, &no_values).plan;
        assert_eq!(named.map(|g| (g.phases, g.open_at)), Some((3, 0.5)));
        for (phases, open_at, problem) in [
            (Some(0), None, "`plan.phases`"),
            (Some(9), None, "`plan.phases`"),
            (None, Some(0.0), "`plan.open_at`"),
            (None, Some(1.5), "`plan.open_at`"),
        ] {
            assert!(
                phased(phases, open_at)
                    .problems()
                    .iter()
                    .any(|e| e.contains(problem)),
                "{phases:?} {open_at:?}"
            );
        }
    }

    #[test]
    fn a_building_rule_needs_its_rule_and_nothing_else_has_one() {
        let rule = |template: &str, within_m: f64| RuleFile {
            template: template.to_owned(),
            within_m,
            walls: Vec::new(),
            gap_m: 0.0,
            from_m: 0.0,
            storeys: 0,
            needs: Some("core:technique/firing".to_owned()),
        };
        let roofs_with = |r: Option<RuleFile>| PolicyFile {
            does: "building_rule".to_owned(),
            answers: vec!["burned".to_owned()],
            levy_shares: Vec::new(),
            relief_days: 0.0,
            rule: r,
            tenure: None,
            forum: None,
            limit: None,
            duty: None,
            privilege: None,
            debts: None,
            lending: None,
            mint: None,
            pay: None,
            medium: None,
            ..file()
        };
        let roofs = roofs_with(Some(rule("roofs_unburnt", 4.0)));
        assert!(roofs.problems().is_empty(), "{:?}", roofs.problems());
        let firing = |id: &str| (id == "core:technique/firing").then_some(3);
        let d = roofs.def(&no_values, &firing);
        assert_eq!(d.kind, PolicyKind::BuildingRule);
        assert_eq!(
            d.rule,
            Some(RuleDef {
                template: RuleTemplate::RoofsUnburnt { within_m: 4.0 },
                needs: Some(3),
            })
        );
        let missing = roofs_with(None);
        assert!(
            missing
                .problems()
                .iter()
                .any(|p| p.contains("needs a `[rule]`"))
        );
        for bad in [
            rule("walls", 4.0),
            rule("roofs_unburnt", 0.0),
            rule("roofs_unburnt", 99.0),
        ] {
            let f = roofs_with(Some(bad));
            assert!(!f.problems().is_empty());
        }
        let stray = PolicyFile {
            rule: Some(rule("roofs_unburnt", 4.0)),
            tenure: None,
            forum: None,
            limit: None,
            duty: None,
            privilege: None,
            debts: None,
            lending: None,
            mint: None,
            pay: None,
            medium: None,
            ..file()
        };
        assert!(
            stray
                .problems()
                .iter()
                .any(|p| p.contains("only a `building_rule`"))
        );
        // Walls of named kinds, and a least gap (content API 94).
        let other = |template: &str, walls: &[&str], gap_m: f64| {
            roofs_with(Some(RuleFile {
                template: template.to_owned(),
                within_m: 0.0,
                walls: walls.iter().map(|w| (*w).to_owned()).collect(),
                gap_m,
                from_m: gap_m,
                storeys: 0,
                needs: None,
            }))
        };
        let walls = other("walls_of_kinds", &["rubble", "mud_brick"], 0.0);
        assert!(walls.problems().is_empty(), "{:?}", walls.problems());
        assert_eq!(
            walls.def(&no_values, &firing).rule.map(|r| r.template),
            Some(RuleTemplate::WallsOfKinds { kinds: 0b0110 })
        );
        assert!(
            !other("walls_of_kinds", &["straw"], 0.0)
                .problems()
                .is_empty()
        );
        assert!(!other("walls_of_kinds", &[], 0.0).problems().is_empty());
        let gap = other("least_gap", &[], 3.0);
        assert!(gap.problems().is_empty(), "{:?}", gap.problems());
        assert_eq!(
            gap.def(&no_values, &firing).rule.map(|r| r.template),
            Some(RuleTemplate::LeastGap { gap_m: 3.0 })
        );
        assert!(!other("least_gap", &[], 0.0).problems().is_empty());
        // Worked fires from others' homes, and middens from wells (content API 148).
        let fires = other("fire_from_homes", &[], 10.0);
        assert!(fires.problems().is_empty(), "{:?}", fires.problems());
        assert_eq!(
            fires.def(&no_values, &firing).rule.map(|r| r.template),
            Some(RuleTemplate::FireFromHomes { from_m: 10.0 })
        );
        let wells = other("midden_from_wells", &[], 30.0);
        assert!(wells.problems().is_empty(), "{:?}", wells.problems());
        assert_eq!(
            wells.def(&no_values, &firing).rule.map(|r| r.template),
            Some(RuleTemplate::MiddenFromWells { from_m: 30.0 })
        );
        assert!(!other("midden_from_wells", &[], 0.0).problems().is_empty());
        assert!(!other("fire_from_homes", &[], 60.0).problems().is_empty());
        // A height rule over named wall kinds, and a ties rule (content API 152).
        let height = |storeys: u8, walls: &[&str]| {
            roofs_with(Some(RuleFile {
                template: "most_storeys".to_owned(),
                within_m: 0.0,
                walls: walls.iter().map(|w| (*w).to_owned()).collect(),
                gap_m: 0.0,
                from_m: 0.0,
                storeys,
                needs: None,
            }))
        };
        let low = height(1, &["rubble", "mud_brick"]);
        assert!(low.problems().is_empty(), "{:?}", low.problems());
        assert_eq!(
            low.def(&no_values, &firing).rule.map(|r| r.template),
            Some(RuleTemplate::MostStoreys {
                storeys: 1,
                kinds: 0b0110
            })
        );
        assert!(!height(0, &["rubble"]).problems().is_empty());
        assert!(!height(4, &["rubble"]).problems().is_empty());
        assert!(!height(1, &[]).problems().is_empty());
        let ties = other("tied_walls", &[], 0.0);
        assert!(ties.problems().is_empty(), "{:?}", ties.problems());
        assert_eq!(
            ties.def(&no_values, &firing).rule.map(|r| r.template),
            Some(RuleTemplate::TiedWalls)
        );
    }

    #[test]
    fn a_template_compiles_with_its_levels_in_order() {
        let f = file();
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        let d = f.def(&no_values, &no_values);
        assert_eq!(d.kind, PolicyKind::CommonStore);
        assert_eq!(d.answers, vec![IssueKind::FoodShort]);
        assert_eq!(d.levy_shares, vec![0.05, 0.1, 0.2]);
    }

    #[test]
    fn unknown_kinds_issues_and_shares_out_of_range_are_refused() {
        let mut f = file();
        f.does = "tithe".to_owned();
        f.answers = vec!["floods".to_owned()];
        assert_eq!(f.problems().len(), 2, "{:?}", f.problems());
        let mut f = file();
        f.levy_shares = vec![0.0, 1.5];
        f.relief_days = 0.0;
        assert_eq!(f.problems().len(), 3, "{:?}", f.problems());
        f.answers.clear();
        assert!(f.problems().iter().any(|m| m.contains("at least one")));
    }

    #[test]
    fn a_keeper_levies_nothing() {
        let mut f = file();
        f.does = "keep_store".to_owned();
        f.answers = vec!["store_unkept".to_owned()];
        assert_eq!(f.problems().len(), 1, "it must not carry levy shares");
        f.levy_shares.clear();
        f.relief_days = 0.0;
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        assert_eq!(f.def(&no_values, &no_values).kind, PolicyKind::KeepStore);
        // So does a watch.
        f.does = "keep_watch".to_owned();
        f.answers = vec!["takings".to_owned()];
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        f.relief_days = 5.0;
        assert!(
            f.problems()
                .iter()
                .any(|m| m.contains("`keep_watch` policy levies nothing"))
        );
    }

    #[test]
    fn a_law_against_taking_carries_its_bundles_and_nothing_else_does() {
        let mut f = file();
        f.does = "against_taking".to_owned();
        f.answers = vec!["takings".to_owned()];
        f.levy_shares.clear();
        f.relief_days = 0.0;
        assert!(
            f.problems().iter().any(|m| m.contains("1 to 8 bundles")),
            "it needs a bundle: {:?}",
            f.problems()
        );
        f.bundles = vec![
            BundleFile {
                compensation_days: 0.0,
                fine_days: 0.0,
                exile: false,
                remove: false,
                yearly_days: 0.0,
            },
            BundleFile {
                compensation_days: 3.0,
                fine_days: 1.5,
                exile: true,
                remove: false,
                yearly_days: 0.0,
            },
        ];
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        let d = f.def(&no_values, &no_values);
        assert_eq!(d.kind, PolicyKind::AgainstTaking);
        assert_eq!(d.answers, vec![IssueKind::Takings]);
        assert_eq!(d.bundles[1].compensation_days, 3.0);
        assert_eq!(d.bundles[1].fine_days, 1.5);
        assert!(d.bundles[1].exile && !d.bundles[0].exile);
        f.bundles[0].fine_days = -1.0;
        f.bundles[1].compensation_days = 400.0;
        assert_eq!(f.problems().len(), 2, "{:?}", f.problems());
        // Only a plan's bundle takes a building down or asks a yearly payment (content API 147).
        f.bundles[0].fine_days = 0.0;
        f.bundles[1].compensation_days = 3.0;
        f.bundles[0].remove = true;
        assert!(
            f.problems()
                .iter()
                .any(|m| m.contains("only a `plan` policy's bundle"))
        );
        f.bundles[0].remove = false;
        // A common store has no bundles.
        let mut g = file();
        g.bundles = f.bundles.clone();
        assert!(
            g.problems()
                .iter()
                .any(|m| m.contains("only an `against_taking`"))
        );
    }

    #[test]
    fn a_curfew_carries_its_hours_and_nothing_else_does() {
        let mut f = file();
        f.does = "curfew".to_owned();
        f.answers = vec!["takings".to_owned()];
        f.levy_shares.clear();
        f.relief_days = 0.0;
        assert!(
            f.problems().iter().any(|m| m.contains("1 to 8 pairs")),
            "{:?}",
            f.problems()
        );
        f.hours = vec![[21, 5]];
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        let d = f.def(&no_values, &no_values);
        assert_eq!(d.kind, PolicyKind::Curfew);
        assert_eq!(d.hours, vec![(21, 5)]);
        f.hours = vec![[24, 5], [6, 6]];
        assert_eq!(f.problems().len(), 2, "{:?}", f.problems());
        // A common store has no hours.
        let mut g = file();
        g.hours = vec![[21, 5]];
        assert!(g.problems().iter().any(|m| m.contains("only a `curfew`")));
    }

    #[test]
    fn an_amendment_combines_its_levels_into_bodies_and_nothing_else_has_them() {
        let mut f = file();
        f.does = "amend_body".to_owned();
        f.answers = vec!["overruled".to_owned()];
        f.levy_shares = Vec::new();
        f.relief_days = 0.0;
        f.members = vec!["adults".to_owned(), "elders".to_owned()];
        f.quorum_shares = vec![0.25, 0.5];
        f.pass = vec!["more_for".to_owned()];
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        let d = f.def(&no_values, &no_values);
        assert_eq!(d.kind, PolicyKind::AmendBody);
        assert_eq!(d.answers, vec![IssueKind::Overruled]);
        assert_eq!(d.bodies.len(), 4);
        assert_eq!(d.bodies[0].members, Membership::Adults);
        assert_eq!(d.bodies[3].members, Membership::Elders);
        assert!((d.bodies[3].quorum_share - 0.5).abs() < 1e-6);
        f.members = vec!["chiefs".to_owned()];
        f.quorum_shares = vec![1.5];
        f.pass = vec!["unanimity".to_owned()];
        assert_eq!(f.problems().len(), 3, "{:?}", f.problems());
        // A common store has none of them.
        let mut g = file();
        g.pass = vec!["more_for".to_owned()];
        assert_eq!(g.problems().len(), 1);
    }
    #[test]
    fn a_mint_needs_its_table_within_bounds_and_only_a_mint_has_one() {
        let mint = |fee: f64| PolicyFile {
            does: "mint".to_owned(),
            answers: vec!["weighed".to_owned()],
            levy_shares: Vec::new(),
            relief_days: 0.0,
            mint: Some(MintFile {
                recipe: "core:recipe/strike_coin".to_owned(),
                mass_g: 8.0,
                fineness: 1.0,
                fee,
            }),
            ..file()
        };
        assert!(
            mint(0.03).problems().is_empty(),
            "{:?}",
            mint(0.03).problems()
        );
        assert!(mint(1.0).problems().iter().any(|p| p.contains("mint.fee")));
        let bare = PolicyFile {
            mint: None,
            ..mint(0.03)
        };
        assert!(
            bare.problems()
                .iter()
                .any(|p| p.contains("needs a `mint` table"))
        );
        let stray = PolicyFile {
            mint: mint(0.03).mint,
            ..file()
        };
        assert!(stray.problems().iter().any(|p| p.contains("`mint` is for")));
        let d = mint(0.03)
            .mint_def(&|id| (id == "core:recipe/strike_coin").then_some(4))
            .expect("resolved");
        assert_eq!(d.recipe, 4);
        assert!((d.metal_kg() - 0.008).abs() < 1e-9);
    }
}
