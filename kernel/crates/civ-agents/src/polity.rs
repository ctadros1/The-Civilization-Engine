//! The polity (ADR-0013): each settlement's body politic, with the founding custom it decides
//! by, its laws and their histories, and its common store. This module is pure: the values, and
//! the forecasts and choices made over copies of the facts. Where people meet, decide, pay and
//! ask is `population::polity`.
//!
//! No issue carries a weight toward a policy (ADR-0013 §5): an issue only makes a move
//! available, and every move, stance and payment is scored by its forecast effect on the
//! person's own household and on those who regard them.

use civ_core::{PermanentId, SimTime};

use crate::person::Flows;

/// What a policy template does, as the kernel carries it out (ADR-0013 §3). Content authors the
/// templates; a law names its template by content id. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyKind {
    /// A share of each household's threshed grain goes into the polity's store, and a household
    /// short of food may ask the store for some (a levy and relief: research 09-17 §1.5, 09-01
    /// §3.3).
    CommonStore,
    /// The one the gathering names keeps the common store under their roof (an office in its
    /// first form, ADR-0013 §2): it spoils as a roofed store does, and those who ask for relief
    /// ask at their home. The law lapses when they die or leave.
    KeepStore,
    /// Taking from another household's store is a wrong the gathering hears (M4b slice AB,
    /// ADR-0015 §4): whoever it finds took gives back what was taken, and the law's bundle
    /// besides (research 09-07 §6.2).
    AgainstTaking,
    /// The one the gathering names keeps watch over the households' stores at night (M4b slice
    /// AC, ADR-0015 §6: a watch is an office, as the storekeeper is): rounds they choose to walk,
    /// guarding and seeing only where they are. The law lapses when they die or leave.
    KeepWatch,
    /// Nobody may be away from home in the hours it sets, but the watch at its rounds and those
    /// at a gathering (M4b slice AD; research 12-04 §1.1, the brief's §1.6): a prohibition whose
    /// keeping is each person's choice (09-06 §1.5), with no sanction in v0.
    Curfew,
    /// The custom itself changes (M4c slice AF; ADR-0013 §2, ADR-0017 §1): who belongs to the
    /// body, how many must come and how it decides, by the custom's own procedure (research
    /// 09-02 §3.7, 09-05 §3.9: who occupies a gate is an amendable rule).
    AmendBody,
    /// A law in force ends, with nothing in its place (M4c slice AI, step two; ADR-0017 §5): the
    /// law it names is superseded, and it is carried rather than in force. A body that took the
    /// deciding weighs ending the laws the custom it replaced made (research 09-11 §1.6:
    /// provisional authority over what survives, unless actors dismantle it).
    Repeal,
    /// The polity claims the places where its people saw people of another settlement at work:
    /// outsiders may use them only by leave (M5c slice AT, ADR-0020 §5). Its own people's use is
    /// unchanged, and nobody is stopped: a claim is words until outsiders heed it.
    ClaimPlace,
    /// The polity's side of an agreement with another polity (M5c slice AU, ADR-0020 §6): its
    /// gathering ratifies what two who met agreed on; it is in force only once both have passed
    /// it and each has heard of the other's decision.
    Agreement,
    /// The polity builds a crossing over a stream its people wade (M5c slice AW, step three;
    /// research 11-07 §4.1: household or village cooperation, labour contributed): it begins once
    /// passed, owned by the polity, and each household is asked an equal share of its work, which
    /// its people give or do not (09-06: labour asked, labour given and the work done are kept
    /// apart). Nobody is made to work, and nothing follows a share not given.
    BuildCrossing,
    /// Nobody may draw water at the source it names, one its sponsor suspects (M6a slice BA,
    /// step four; ADR-0021 §7): a prohibition whose keeping is each person's choice, as a
    /// curfew's is (research 09-06 §1.5), with no sanction in v0. Someone with no other source
    /// they know draws there still (need over the gathering's word, as over a claim).
    CloseSource,
    /// The polity digs a well beside the hearth (M6a slice BA, step four; ADR-0021 §7): it begins
    /// once passed, the polity's, and each household is asked an equal share of its work, as a
    /// crossing built together is (09-06: labour asked and labour given are kept apart); once
    /// open, the polity's people may draw at it. Nobody is made to work.
    DigWell,
    /// The polity builds a flood bank where it would have kept out the water that cost its
    /// households (M6b slice BG; ADR-0022 §11): it begins once passed, the polity's, and each
    /// household is asked an equal share of its work, as a crossing built together is (research
    /// 12-08 §1.2: those who gain and those who work may differ). Nobody is made to work, and
    /// nobody keeps it up unless a law asks (ADR-0022 §11).
    BuildBank,
    /// The polity keeps a flood bank it built (M6b slice BG; ADR-0022 §11: nobody mends a
    /// polity's bank unless a law asks; research 12-08 §1.4: maintenance needs an organization
    /// of its own, not the mobilization that built it): the bank is looked over when a high water
    /// goes down and each year, and whenever it wants mending each household is asked an equal
    /// share of the work, as for building it. Nobody is made to work.
    KeepBank,
    /// A rule for building (M6b slice BH; ADR-0022 §12): what its template requires of buildings
    /// begun from the day it is in force ([`RuleDef`]). Old buildings stand as they are (research
    /// 11-09 §1.7: never rebuild the whole village when a law passes). Keeping it is each
    /// builder's choice, as a curfew's is, with no sanction in M6b.
    BuildingRule,
    /// The polity sends a band against the settlement whose people raided its households (M6c
    /// slice BJ, step three; ADR-0023 §5): the one it names leads it, and each household owes
    /// one of its grown people to go, whom it chooses, or keeps its share back as a levy is kept
    /// back. The band goes once; when it is home, or never set off, the law has run its course.
    SendBand,
    /// A raid by the polity's own people on another settlement is a wrong its gathering hears
    /// (M6c slice BJ, step three; ADR-0023 §5): whoever it finds led one gives back what was
    /// taken, and the law's bundle besides, to the household raided.
    AgainstRaiding,
    /// The polity raises a palisade round its hearth, taking in the homes within a reach (M6c
    /// slice BK, step two; ADR-0023 §6): it is begun once passed, the polity's, and each
    /// household is asked an equal share of its work, as a crossing built together is; homes
    /// outside the line pay without its cover. Nobody is made to work.
    Enclose,
    /// The polity takes in the settlement a band its law sent holds (M6c slice BM; ADR-0023
    /// §9): the held polity's deciding closes, its laws lapse, its store becomes this polity's
    /// and its residents become this polity's members. Passed, it is done at once and has run its
    /// course.
    Annex,
    /// A forum (M7b slice BR, ADR-0025 §1): who hears the cases of the kinds of law its template
    /// names, besides the gathering: one holder it names, a panel drawn by lot, or the elders.
    HearCases,
    /// A limit (M7b slice BT, ADR-0025 §3): a law above laws, forbidding laws whose fields meet a
    /// predicate the kernel knows; adopted only through the custom's amendment route, and changed
    /// only by its own rule. A law that breaches it still passes: it matters only when someone
    /// challenges and a forum with review finds against the law.
    Limit,
    /// A duty (M7c slice BV, ADR-0026 §1): a member's household that buys goods of its template's
    /// purpose at another settlement's door owes the polity's store a share of them (the law's
    /// levy share), paid at home in kind as the levy is, or kept back.
    Duty,
    /// An embargo (M7c slice BV, ADR-0026 §1): no member may buy at the doors of the settlement
    /// it names (its target). Keeping it is each person's choice, as a curfew's is.
    Embargo,
    /// Recognition (M7c slice BW, ADR-0026 §2): the polity recognizes the organization by stake
    /// it names (its target). In BW a recognized organization's members' households owe none of
    /// the polity's duties on goods of the organization's purpose; privileges follow in BX. It
    /// ends with the organization.
    Recognize,
    /// A privilege (M7c slice BX, ADR-0026 §3): the organization it names (its target) alone may
    /// offer goods of its template's purpose at doors in the settlement (a licence), or alone may
    /// sell there such goods fetched from another settlement (a staple). Keeping it is each
    /// seller's choice; a breach is an offer at a door, which becomes a case only if someone
    /// brings it, and a finding imposes the template's bundle.
    Privilege,
    /// Debts heard (M7d slice CB, ADR-0027 §3): a creditor whose loan was refused or left unpaid
    /// when its days of grace ran out may bring it before the gathering, or a forum that hears
    /// such cases; a finding imposes the template's remedy ([`DebtsDef`]). Without it, a
    /// creditor has only its own answers and its household's memory.
    DebtsHeard,
    /// Lending from the store (M7d slice CC, ADR-0027 §4): a household of the polity short of
    /// food before its harvest may borrow from the common store at the law's terms
    /// ([`LendingDef`]), repaid into it after the harvest. Relief stays a gift.
    LendFromStore,
    /// A mint (M7d slice CE, ADR-0027 §6): the one the gathering names strikes coin, as its
    /// template sets ([`MintDef`]), from the metal households of the polity bring, the polity's
    /// store keeping the template's fee in coin. The law lapses when they die or leave.
    Mint,
    /// Pay for common work (M7d slice CE3a, ADR-0027 §6: the polity's first spending): the
    /// polity's store pays the good its template names ([`PayDef`]), so much an hour, to each
    /// household for the hours it gives to the polity's works, as far as the store holds it.
    PayWork,
    /// A way built together (M8b slice CQ, third part; ADR-0029 §6): a passing place cleared at
    /// the middle of a single-cart stretch where the settlement's carts wait for each other, as a
    /// flood bank is built together: its site where those waits would repay the work over the
    /// place's life, equal shares of the work, payable by `pay_work`.
    BuildWay,
    /// A plan laid over the ground (M8c slice CT, second part; ADR-0030 §2): a template's grid
    /// of streets, a square and parcels laid where the sponsor chose, saved as realised; in
    /// force, it reserves its streets and square (ADR-0030 §3). An amendment is a new version.
    Plan,
    /// A building raised together (M8d slice DC, ADR-0031 §6), shaped as `build_way`: the
    /// program its template names ([`RaiseDef`]) answering a loss the polity measured, put up
    /// near the hearth (or on a plan's square), each household asked an equal share of its work,
    /// payable by `pay_work`; the polity owns it.
    RaiseBuilding,
    /// A line of track laid together (M10b slice DX, third part), shaped as `build_way`: from a
    /// pit many of the settlement's households dig to beside its hearth, where their dig days
    /// would repay its laying over a wagon's life, each household asked an equal share of its
    /// work and timber; the polity keeps it.
    BuildLine,
    /// Streets lit at night (M10b slice DY, fourth part, third step; ADR-0034 §9): an office, as
    /// a watch is, whose holder lights lamps the polity sets on its ways beside its homes and puts
    /// them out on a lamplighter's rounds; lit streets let those about see after dark.
    LightStreets,
    /// A school kept (M10c slice EB, third part; ADR-0035 §5): an office whose holder takes the
    /// children of the polity's households to learn a technique they know without a fee, and is
    /// paid for the hours taught where work is paid.
    KeepSchool,
    /// A licence to practise (M10c slice EC, third part; ADR-0035 §6; ADR-0026 §3; brief
    /// m10-services §3.4): only those who know its technique may be paid to tend the sick; kin,
    /// who ask no fee, are untouched.
    Licence,
    /// The sick kept apart (M10c slice EE, first part; ADR-0035 §7; brief m10-services §2.4;
    /// research 05-05 §1.6-1.7): a household with someone ill keeps to its home, and others keep
    /// from it, until some days after the last is well; kept by choice, as a curfew is.
    KeepApart,
    /// A register of the sick kept (M10c slice EE, second part; ADR-0035 §7; brief m10-services
    /// §2.4): an office whose holder must know how to write, who enters in it the sickness and
    /// the deaths of disease of the polity's households they hear of; their tallies of where
    /// sickness comes from run over it, a wider record than anyone's own.
    KeepRegister,
    /// Middens cleared from the water (M10c slice EE, third part; ADR-0035 §7; brief
    /// m10-services §2.4): a household whose home lies near an open well or a stream carries its
    /// midden away, to a place as far from every source; kept by choice.
    ClearMiddens,
}

impl PolicyKind {
    /// Every kind, in code order.
    pub const ALL: [PolicyKind; 39] = [
        PolicyKind::CommonStore,
        PolicyKind::KeepStore,
        PolicyKind::AgainstTaking,
        PolicyKind::KeepWatch,
        PolicyKind::Curfew,
        PolicyKind::AmendBody,
        PolicyKind::Repeal,
        PolicyKind::ClaimPlace,
        PolicyKind::Agreement,
        PolicyKind::BuildCrossing,
        PolicyKind::CloseSource,
        PolicyKind::DigWell,
        PolicyKind::BuildBank,
        PolicyKind::KeepBank,
        PolicyKind::BuildingRule,
        PolicyKind::SendBand,
        PolicyKind::AgainstRaiding,
        PolicyKind::Enclose,
        PolicyKind::Annex,
        PolicyKind::HearCases,
        PolicyKind::Limit,
        PolicyKind::Duty,
        PolicyKind::Embargo,
        PolicyKind::Recognize,
        PolicyKind::Privilege,
        PolicyKind::DebtsHeard,
        PolicyKind::LendFromStore,
        PolicyKind::Mint,
        PolicyKind::PayWork,
        PolicyKind::BuildWay,
        PolicyKind::Plan,
        PolicyKind::RaiseBuilding,
        PolicyKind::BuildLine,
        PolicyKind::LightStreets,
        PolicyKind::KeepSchool,
        PolicyKind::Licence,
        PolicyKind::KeepApart,
        PolicyKind::KeepRegister,
        PolicyKind::ClearMiddens,
    ];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            PolicyKind::CommonStore => "common_store",
            PolicyKind::KeepStore => "keep_store",
            PolicyKind::AgainstTaking => "against_taking",
            PolicyKind::KeepWatch => "keep_watch",
            PolicyKind::Curfew => "curfew",
            PolicyKind::AmendBody => "amend_body",
            PolicyKind::Repeal => "repeal",
            PolicyKind::ClaimPlace => "claim_place",
            PolicyKind::Agreement => "agreement",
            PolicyKind::BuildCrossing => "build_crossing",
            PolicyKind::CloseSource => "close_source",
            PolicyKind::DigWell => "dig_well",
            PolicyKind::BuildBank => "build_bank",
            PolicyKind::KeepBank => "keep_bank",
            PolicyKind::BuildingRule => "building_rule",
            PolicyKind::SendBand => "send_band",
            PolicyKind::AgainstRaiding => "against_raiding",
            PolicyKind::Enclose => "enclose",
            PolicyKind::Annex => "annex",
            PolicyKind::HearCases => "hear_cases",
            PolicyKind::Limit => "limit",
            PolicyKind::Duty => "duty",
            PolicyKind::Embargo => "embargo",
            PolicyKind::Recognize => "recognize",
            PolicyKind::Privilege => "privilege",
            PolicyKind::DebtsHeard => "debts_heard",
            PolicyKind::LendFromStore => "lend_from_store",
            PolicyKind::Mint => "mint",
            PolicyKind::PayWork => "pay_work",
            PolicyKind::BuildWay => "build_way",
            PolicyKind::Plan => "plan",
            PolicyKind::RaiseBuilding => "raise_building",
            PolicyKind::BuildLine => "build_line",
            PolicyKind::LightStreets => "light_streets",
            PolicyKind::KeepSchool => "keep_a_school",
            PolicyKind::Licence => "licence",
            PolicyKind::KeepApart => "keep_apart",
            PolicyKind::KeepRegister => "keep_register",
            PolicyKind::ClearMiddens => "clear_middens",
        }
    }

    /// The kind with an authored name.
    pub fn from_name(name: &str) -> Option<PolicyKind> {
        PolicyKind::ALL.into_iter().find(|k| k.name() == name)
    }
}

/// A recurring problem a polity can face (research 09-05 §1.1). An issue only makes moves
/// available; it weighs toward none. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueKind {
    /// The settlement's food ran short within the last year, or a household's food will not last
    /// until its next harvest: a shortfall against the outlook.
    FoodShort,
    /// A common store is in force and holds food, and nobody keeps it under a roof.
    StoreUnkept,
    /// A household of the settlement found food taken from its store within the last year (M4b
    /// slice AB): what its members know, not what happened.
    Takings,
    /// A gathering they came to within memory decided against where they stood, or they hold a
    /// grievance against the gathering (M4c slice AF): what one person has seen and holds, so
    /// it opens moves to them alone.
    Overruled,
    /// A faction petitioned the gathering for it (M4c slice AH): its members held grievances
    /// against the gathering. It answers no settlement's issue and opens no move to anyone.
    Petition,
    /// The custom was taken from the gathering lately (M4c slice AI, step two): the body that took
    /// it weighs which of the laws the old custom made stand (research 09-11 §1.6). It opens
    /// moves to that body's members, for those laws alone.
    Founding,
    /// People of another settlement worked places a household of the settlement works, within
    /// the last year, at places it has not claimed (M5c slice AT): what its members saw.
    Outsiders,
    /// A household of the settlement heard that another polity claims places its people work,
    /// which its own polity does not claim (M5c slice AU): what its members were told.
    ClaimedFromUs,
    /// One of the settlement's people met someone of another settlement who sought terms
    /// between their polities, and the two agreed on some to put to their gatherings (M5c slice
    /// AU). It answers no settlement's issue and opens no move to anyone.
    TermsSought,
    /// A household of the settlement wades a stream where a crossing of a system its people
    /// know could stand and none does (M5c slice AW, step three): what its members walked.
    Fords,
    /// Someone of a household of the settlement lay sick at home lately, as its members saw (M6a
    /// slice BA, step four; ADR-0021 §7): what they hold as news of their own.
    Sickness,
    /// Someone holds that a source of water sickens, from their own tally (M6a slice BA, step
    /// four): what one person holds, so it opens moves to them alone, as `Overruled` does.
    SourceSuspected,
    /// A household of the settlement walks far for water, as it last walked it (so far that its
    /// people use less water for the walk), and a well dug together beside the hearth would save
    /// the settlement's households more walking over its lining's life than its work (M6a slice
    /// BA, step four), as fords make a crossing an issue.
    WaterFar,
    /// A flood cost a household of the settlement within its people's memory, as they saw (its
    /// stores spoiled where water stood, its sown crop drowned), and a bank built together would
    /// have kept that water out, its saving over its life above its work (M6b slice BG), as
    /// fords make a crossing an issue.
    Flooded,
    /// A flood bank the polity built wants mending, and no law of its keeps it (M6b slice BG).
    BankWorn,
    /// A household of the settlement lost a building to fire, or its people saw one burn, within
    /// their memory (M6b slice BH; ADR-0022 §12).
    Burned,
    /// A building of the settlement gave way within its people's memory, as they heard told
    /// (M6b slice BH; ADR-0022 §12).
    GaveWay,
    /// A household of the settlement knows people of another settlement took from its store
    /// within its people's memory (M6c slice BI; ADR-0023 §2).
    Raided,
    /// A band the polity's law sent holds another settlement (M6c slice BM; ADR-0023 §9).
    Holding,
    /// A case a household of the settlement brought within memory was not heard, or was not
    /// found (M7b slice BR, ADR-0025 §1): what a forum answers.
    CasesWaiting,
    /// A household of the settlement offers at home goods its people fetched from another
    /// settlement lately (M7c slice BV, ADR-0026 §1): what a duty or an embargo answers.
    FetchedElsewhere,
    /// A loan their household made was refused or left unpaid, and nothing was taken for it
    /// (M7d slice CB, ADR-0027 §3): what a law of debts heard answers.
    DebtsUnpaid,
    /// A household of the settlement paid or was paid in weighed metal within its market's
    /// memory (M7d slice CE, ADR-0027 §6): what a mint answers.
    Weighed,
    /// The polity's store holds money while a law of its asks households to work on its common
    /// works (M7d slice CE3a): what paying for that work answers.
    CoinIdle,
    /// Carts of the settlement's households wait for each other at a stretch of trail one cart
    /// wide, as they remember, and a passing place cleared there together would spare them more
    /// waiting over its life than its work (M8b slice CQ, third part), as fords make a crossing
    /// an issue.
    CartsWait,
    /// A household of the settlement found no clear ground for the home it meant to build
    /// within its reach (M8c slice CT, second part; ADR-0030 §2).
    NoGround,
    /// A building was begun on a worn trail of the settlement's (M8c slice CT, second part).
    WayBuiltOver,
    /// A household built beyond the settlement's built edge (M8c slice CT, second part).
    NewGround,
    /// The common store spoiled within the years its polity measures, and no storehouse of the
    /// polity's stands or is being raised (M8d slice DC, ADR-0031 §6): what raising one answers.
    StoreSpoils,
    /// A gathering held in the open came to nothing for want of a quorum on a wet day, within the
    /// years its polity measures, and no hall of the polity's stands or is being raised (M8d
    /// slice DC): what raising one answers.
    RainedOff,
    /// A fire burned awhile before anyone saw it, within the years its polity measures, while
    /// its watch keeps watch and no tower of the polity's stands or is being raised (M8d slice
    /// DC): what raising one answers.
    FiresUnseen,
    /// Several of the settlement's households dig one pit and carry what they dig home by hand
    /// or push it alone, where a line laid together would spare them more than its laying (M10b
    /// slice DX, third part): what `build_line` answers.
    LoadsCarried,
    /// Something was taken from a household of the settlement after dark, unseen, within memory,
    /// and someone of it knows how to make a light (M10b slice DY, fourth part, third step): what
    /// `light_streets` answers.
    DarkTakings,
    /// A household of the polity could not hold the fee to send a child to learn a technique
    /// it lacked within a year, and someone of the settlement knows it (M10c slice EB, third
    /// part): what `keep_a_school` answers.
    Untaught,
    /// Within the year someone of a household of the polity died in the care of one it paid who
    /// knew no treatment of their sickness, and someone grown of the settlement knows one (M10c
    /// slice EC, third part): what `licence` answers.
    UnknowingCare,
    /// Within the year someone of a household of the polity died of a sickness that passes from
    /// the sick to those near them, at home or in company (M10c slice EE, first part): what
    /// `keep_apart` answers.
    Contagion,
    /// Severe cases of the polity's households lay ill with nobody tending them, measured, and no
    /// sickhouse of its stands or is being raised (M10c slice EE, fourth part): what raising a
    /// sickhouse answers.
    Untended,
}

impl IssueKind {
    /// Every kind, in code order.
    pub const ALL: [IssueKind; 37] = [
        IssueKind::FoodShort,
        IssueKind::StoreUnkept,
        IssueKind::Takings,
        IssueKind::Overruled,
        IssueKind::Petition,
        IssueKind::Founding,
        IssueKind::Outsiders,
        IssueKind::ClaimedFromUs,
        IssueKind::TermsSought,
        IssueKind::Fords,
        IssueKind::Sickness,
        IssueKind::SourceSuspected,
        IssueKind::WaterFar,
        IssueKind::Flooded,
        IssueKind::BankWorn,
        IssueKind::Burned,
        IssueKind::GaveWay,
        IssueKind::Raided,
        IssueKind::Holding,
        IssueKind::CasesWaiting,
        IssueKind::FetchedElsewhere,
        IssueKind::DebtsUnpaid,
        IssueKind::Weighed,
        IssueKind::CoinIdle,
        IssueKind::CartsWait,
        IssueKind::NoGround,
        IssueKind::WayBuiltOver,
        IssueKind::NewGround,
        IssueKind::StoreSpoils,
        IssueKind::RainedOff,
        IssueKind::FiresUnseen,
        IssueKind::LoadsCarried,
        IssueKind::DarkTakings,
        IssueKind::Untaught,
        IssueKind::UnknowingCare,
        IssueKind::Contagion,
        IssueKind::Untended,
    ];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            IssueKind::FoodShort => "food_short",
            IssueKind::StoreUnkept => "store_unkept",
            IssueKind::Takings => "takings",
            IssueKind::Overruled => "overruled",
            IssueKind::Petition => "petition",
            IssueKind::Founding => "founding",
            IssueKind::Outsiders => "outsiders",
            IssueKind::ClaimedFromUs => "claimed_from_us",
            IssueKind::TermsSought => "terms_sought",
            IssueKind::Fords => "fords",
            IssueKind::Sickness => "sickness",
            IssueKind::SourceSuspected => "source_suspected",
            IssueKind::WaterFar => "water_far",
            IssueKind::Flooded => "flooded",
            IssueKind::BankWorn => "bank_worn",
            IssueKind::Burned => "burned",
            IssueKind::GaveWay => "gave_way",
            IssueKind::Raided => "raided",
            IssueKind::Holding => "holding",
            IssueKind::CasesWaiting => "cases_waiting",
            IssueKind::FetchedElsewhere => "fetched_elsewhere",
            IssueKind::DebtsUnpaid => "debts_unpaid",
            IssueKind::Weighed => "weighed",
            IssueKind::CoinIdle => "coin_idle",
            IssueKind::CartsWait => "carts_wait",
            IssueKind::NoGround => "no_ground",
            IssueKind::WayBuiltOver => "way_built_over",
            IssueKind::NewGround => "new_ground",
            IssueKind::StoreSpoils => "store_spoils",
            IssueKind::RainedOff => "rained_off",
            IssueKind::FiresUnseen => "fires_unseen",
            IssueKind::LoadsCarried => "loads_carried",
            IssueKind::DarkTakings => "dark_takings",
            IssueKind::Untaught => "untaught",
            IssueKind::UnknowingCare => "unknowing_care",
            IssueKind::Contagion => "contagion",
            IssueKind::Untended => "untended",
        }
    }

    /// The kind with an authored name.
    pub fn from_name(name: &str) -> Option<IssueKind> {
        IssueKind::ALL.into_iter().find(|k| k.name() == name)
    }

    /// In words, after "because": "food would not last until the harvest".
    pub fn words(self) -> &'static str {
        match self {
            IssueKind::FoodShort => "food would not last until the harvest",
            IssueKind::StoreUnkept => "the common store lay in the open with nobody to keep it",
            IssueKind::Takings => "food was taken from households' stores",
            IssueKind::Overruled => "the gathering had decided against them",
            IssueKind::Petition => {
                "those who came to petition for it held it against the gathering"
            }
            IssueKind::Founding => {
                "the deciding had been taken from the gathering, and the laws it made stood only \
                 until the new body weighed them"
            }
            IssueKind::Outsiders => "people of another settlement worked the places theirs did",
            IssueKind::ClaimedFromUs => "another settlement claims places theirs work",
            IssueKind::TermsSought => {
                "one of theirs met someone of another settlement who sought terms with them"
            }
            IssueKind::Fords => "its people waded a stream a crossing could span",
            IssueKind::Sickness => "its people had sickness at home lately",
            IssueKind::SourceSuspected => "they hold that the water there sickens",
            IssueKind::WaterFar => "its people walked far for water",
            IssueKind::Flooded => "floodwater had cost its households",
            IssueKind::BankWorn => "the bank they built against floodwater wanted mending",
            IssueKind::Burned => "fire had taken buildings they knew",
            IssueKind::GaveWay => "buildings they knew had given way",
            IssueKind::Raided => "people of another settlement took from households' stores",
            IssueKind::Holding => "a band of theirs held another settlement",
            IssueKind::CasesWaiting => "a case they brought went unheard or was not found",
            IssueKind::FetchedElsewhere => {
                "people of theirs fetched from elsewhere goods their households offer at home"
            }
            IssueKind::DebtsUnpaid => "a loan their household made was not repaid",
            IssueKind::Weighed => "their households weighed out metal to pay",
            IssueKind::CoinIdle => "the store held money while their common work wanted hands",
            IssueKind::CartsWait => {
                "their carts waited for each other where the way is one cart wide"
            }
            IssueKind::NoGround => "their household found no clear ground to build its home on",
            IssueKind::WayBuiltOver => "a building went up on a trail they walked",
            IssueKind::NewGround => "their households built beyond the edge of the settlement",
            IssueKind::StoreSpoils => "the common store spoiled with no storehouse to keep it",
            IssueKind::RainedOff => {
                "a gathering in the open came to nothing in the rain, with no hall to meet in"
            }
            IssueKind::FiresUnseen => {
                "a fire burned awhile before anyone saw it, with no tower to watch from"
            }
            IssueKind::LoadsCarried => {
                "they carried home what they dug at a pit many of them dig, with no line to it"
            }
            IssueKind::DarkTakings => {
                "something was taken from them in the dark, and nobody saw who took it"
            }
            IssueKind::Untaught => {
                "a child of theirs could not be sent to learn a craft for want of the fee"
            }
            IssueKind::UnknowingCare => {
                "one of theirs died in the care of someone they paid who knew no cure for it"
            }
            IssueKind::Contagion => {
                "one of theirs died of a sickness that passes from the sick to those near them"
            }
            IssueKind::Untended => "those severely ill among them lay with nobody to tend them",
        }
    }
}

/// What a law against taking adds to giving back what was taken (research 09-07 §1.2, §6.2:
/// restitution, compensation and a fine are different transfers, and a punishment is a bundle),
/// in days of the taker's household's food (09-07 §2.3).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sanction {
    /// Food to the household taken from, besides what was taken, days.
    pub compensation_days: f32,
    /// Food to the polity's store, days.
    pub fine_days: f32,
    /// The one found to have taken is sent from the valley (ADR-0015 §5: exile, recorded as a
    /// migration; 09-07 §6.2).
    pub exile: bool,
    /// For a plan (M8c slice CV, ADR-0030 §5): the building found standing on its reserved
    /// ground is taken down.
    pub remove: bool,
    /// For a plan: the building found may stand for this many days of its household's food a
    /// year to the polity's store (regularisation), 0 for none.
    pub yearly_days: f32,
}

impl Sanction {
    /// In words, after "what was taken given back": ", with three days' food to the household
    /// taken from and three to the common store", or "" for nothing more.
    pub fn words(&self) -> String {
        let days = |d: f32| {
            let d = f64::from(d);
            let n = match d.round() as i64 {
                1 => "one",
                2 => "two",
                3 => "three",
                4 => "four",
                5 => "five",
                6 => "six",
                7 => "seven",
                10 => "ten",
                _ => "",
            };
            let unit = if (d - 1.0).abs() < 1e-6 {
                "day's"
            } else {
                "days'"
            };
            if n.is_empty() || (d - d.round()).abs() > 1e-6 {
                format!("{d:.1} {unit}")
            } else {
                format!("{n} {unit}")
            }
        };
        let mut parts = Vec::new();
        if self.compensation_days > 0.0 {
            parts.push(format!(
                "{} food to the household taken from",
                days(self.compensation_days)
            ));
        }
        if self.fine_days > 0.0 {
            parts.push(format!("{} to the common store", days(self.fine_days)));
        }
        let with = match parts.len() {
            0 => String::new(),
            1 => format!(", with {}", parts[0]),
            _ => format!(", with {} and {}", parts[0], parts[1]),
        };
        if self.exile {
            format!("{with}, and is sent from the valley")
        } else {
            with
        }
    }
}

/// A policy template (content kind `policy`, ADR-0013 §3): what it does, the issues whose
/// presence makes proposing it a move, and the parameters a sponsor may put forward.
#[derive(Clone, Debug, PartialEq)]
pub struct PolicyDef {
    /// Content id: what laws refer to.
    pub id: String,
    /// Display name.
    pub name: String,
    /// What it is, in a sentence.
    pub description: String,
    /// What the kernel does under it.
    pub kind: PolicyKind,
    /// The issues whose presence makes proposing it a move.
    pub answers: Vec<IssueKind>,
    /// The shares of threshed grain a sponsor may propose for the levy, smallest first (research
    /// 09-05 §2.3: a sponsor weighs three to eight levels).
    pub levy_shares: Vec<f64>,
    /// The most food one ask brings a household from the store, in days of its need.
    pub relief_days: f64,
    /// For a law against taking: the bundles a sponsor may propose, lightest first.
    pub bundles: Vec<Sanction>,
    /// For a curfew: the hours a sponsor may propose, each `(from, to)` hours of the day, the
    /// curfew running from the first to the second, past midnight when it is the smaller.
    pub hours: Vec<Hours>,
    /// For an amendment of the custom: the bodies a sponsor may propose (M4c slice AF).
    pub bodies: Vec<Body>,
    /// The question people take positions on, in words ("whether to keep a common store"), or
    /// none (M4c slice AG, ADR-0016 §4).
    pub question: Option<String>,
    /// How a law of it bears on each value content names: (index in the catalog's values, −1
    /// against to 1 for), in value order (M4c slice AG, step three).
    pub bears: Vec<(u16, f32)>,
    /// For a rule for building: what it requires (M6b slice BH).
    pub rule: Option<RuleDef>,
    /// For a building raised together: what it raises (M8d slice DC).
    pub raise: Option<RaiseDef>,
    /// For an office (the store's keeper, the watch): how long it is held and how it is filled,
    /// when not until the holder dies or leaves and named by the body (M7a slice BN, ADR-0024 §2).
    pub tenure: Option<Tenure>,
    /// For a forum: who sits and what it hears (M7b slice BR, ADR-0025 §1).
    pub forum: Option<crate::forum::ForumDef>,
    /// For a limit: what it forbids and the rule that changes it (M7b slice BT).
    pub limit: Option<crate::forum::LimitDef>,
    /// For a duty: what it is laid on (M7c slice BV, ADR-0026 §1).
    pub duty: Option<DutyDef>,
    /// For a privilege: what it grants (M7c slice BX, ADR-0026 §3).
    pub privilege: Option<PrivilegeDef>,
    /// For debts heard: what a finding does (M7d slice CB, ADR-0027 §3).
    pub debts: Option<DebtsDef>,
    /// For lending from the store: its terms (M7d slice CC, ADR-0027 §4).
    pub lending: Option<LendingDef>,
    /// For a mint: what it strikes (M7d slice CE, ADR-0027 §6).
    pub mint: Option<MintDef>,
    /// For pay for common work: what an hour is paid (M7d slice CE3a).
    pub pay: Option<PayDef>,
    /// For a common store: a money its levy may be paid in, and at what rate (M7d slice CE3b).
    pub medium: Option<MediumDef>,
    /// For a plan: its template (M8c slice CT, second part).
    pub plan: Option<PlanDef>,
    /// For a licence to practise: the technique it asks (M10c slice EC, third part).
    pub licence: Option<LicenceDef>,
    /// For keeping the sick apart: its terms (M10c slice EE, first part).
    pub keep_apart: Option<KeepApartDef>,
    /// For an office: a technique its holder must know, by index in the catalog's techniques
    /// (M10c slice EE, second part; ADR-0035 §6: an office or post may require a technique).
    pub holder_needs: Option<usize>,
    /// For a register of the sick: what a household believes it spares (M10c slice EE, second
    /// part).
    pub register: Option<RegisterDef>,
    /// For clearing middens from the water: its terms (M10c slice EE, third part).
    pub middens: Option<MiddensDef>,
}

/// The terms of a law clearing middens from the water (M10c slice EE, third part): how far from
/// every open well and stream a household near one carries its heap, metres; how many days a
/// heap carried lies there before it is the household's at home again; the hours a carrying
/// takes and the share of its deaths of disease within the year a household believes cleaner
/// water would have spared (design priors it weighs).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiddensDef {
    pub distance_m: f32,
    pub keep_days: u16,
    pub carry_h: f32,
    pub avoided_share: f32,
}

/// What a household believes a register of the sick spares it (M10c slice EE, second part): the
/// share of its deaths of disease within the year it believes a register would have avoided (a
/// design prior).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegisterDef {
    pub avoided_share: f32,
}

/// How long a law keeping the sick apart keeps a household apart, and what keeping it costs
/// (M10c slice EE, first part): the days after the last of it is well, and the share of a day's
/// work each grown member loses while it keeps to its home (a design prior a household weighs).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeepApartDef {
    pub days_after: u16,
    pub cost_share: f32,
}

/// A money a common store's levy may be paid in, in place of grain (M7d slice CE3b, ADR-0027 §6;
/// research 08-06 §1.4: an obligation's acceptable media and conversion schedule, stored
/// explicitly).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MediumDef {
    /// The money, by index in the catalog's goods.
    pub good: usize,
    /// Units of it that pay for a kilogram of the grain owed.
    pub per_kg: f32,
}

/// What a licence to practise asks of those paid to tend the sick (M10c slice EC, third part):
/// a technique, by index in the catalog's techniques, and its name in words.
#[derive(Clone, Debug, PartialEq)]
pub struct LicenceDef {
    pub technique: usize,
    pub what: String,
}

/// What the store pays for an hour given to the polity's common works under a law of pay for
/// common work (M7d slice CE3a, ADR-0027 §6).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PayDef {
    /// The good paid, by index in the catalog's goods: a money.
    pub good: usize,
    /// Units of it an hour.
    pub per_hour: f32,
}

/// What a mint strikes (M7d slice CE, ADR-0027 §6; research 08-06 §1.5): its recipe (metal and
/// fuel in, the coin good out), and each piece of the issues struck under the law.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MintDef {
    /// The recipe struck by, by index in the catalog's recipes: its first input is the metal,
    /// its first output the coin good.
    pub recipe: usize,
    /// Grams a piece weighs.
    pub mass_g: f32,
    /// The share of a piece that is the metal.
    pub fineness: f32,
    /// The share of the pieces struck from what a household brings that the polity's store
    /// keeps.
    pub fee: f32,
}

impl MintDef {
    /// Kilograms of the metal in a piece.
    pub fn metal_kg(&self) -> f64 {
        f64::from(self.mass_g) * f64::from(self.fineness) / 1000.0
    }
}

/// The terms the common store lends on under a law of lending from the store (M7d slice CC,
/// ADR-0027 §4): set by the law, not by what anyone believes of the borrower.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LendingDef {
    /// The increment owed on what is lent, a share of it.
    pub increment: f32,
    /// The most of the food the store holds it lends at once, a share.
    pub share: f32,
}

/// What a finding under a law of debts heard does (M7d slice CB, ADR-0027 §3): the custom's
/// choice. Codes are part of content: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebtRemedy {
    /// The debt stands, and the debtor is given time: the loan is open again, due `extend_days`
    /// after the finding (an extension in a lean year, research 08-20 §1.6).
    Time = 0,
    /// What is still owed is imposed as a finding's obligation, met as a finding is (paid,
    /// refused and held against, or taken by the watch where a law sends it).
    Imposed = 1,
}

impl DebtRemedy {
    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            DebtRemedy::Time => "time",
            DebtRemedy::Imposed => "imposed",
        }
    }

    /// The remedy with an authored name.
    pub fn from_name(name: &str) -> Option<DebtRemedy> {
        [DebtRemedy::Time, DebtRemedy::Imposed]
            .into_iter()
            .find(|r| r.name() == name)
    }
}

/// A law of debts heard's terms (M7d slice CB, ADR-0027 §3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DebtsDef {
    /// What a finding does.
    pub remedy: DebtRemedy,
    /// The share of what is still owed households expect a finding to move between creditor and
    /// debtor, for their forecasts (all of it when imposed; less for time).
    pub share: f32,
    /// For time: the days after the finding the loan falls due again.
    pub extend_days: u32,
}

/// What a duty is laid on (M7c slice BV, ADR-0026 §1): goods of one purpose bought at another
/// settlement's door, and whether households that offer such goods at home are spared it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DutyDef {
    pub purpose: crate::params::GoodUse,
    pub exempt_sellers: bool,
    /// The law names one who collects it (M7c slice BY, ADR-0026 §4): they see purchases and
    /// assess them, and may ask a payment to assess less.
    pub collector: bool,
}

impl DutyDef {
    /// What it is laid on, in words: "tools bought at another settlement's door".
    pub fn words(&self) -> String {
        format!(
            "{} bought at another settlement's door{}",
            self.purpose.plural_words(),
            if self.exempt_sellers {
                ", but by households that offer them at home"
            } else {
                ""
            }
        )
    }
}

/// What a privilege grants (M7c slice BX, ADR-0026 §3). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grant {
    /// The sole right to offer goods of its purpose at doors in the settlement.
    Licence = 0,
    /// The sole right to sell in the settlement goods of its purpose fetched from another.
    Staple = 1,
}

impl Grant {
    /// Every grant, in code order.
    pub const ALL: [Grant; 2] = [Grant::Licence, Grant::Staple];

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            Grant::Licence => "licence",
            Grant::Staple => "staple",
        }
    }

    /// The grant with an authored name.
    pub fn from_name(name: &str) -> Option<Grant> {
        Grant::ALL.into_iter().find(|g| g.name() == name)
    }
}

/// A privilege's terms (M7c slice BX, ADR-0026 §3): what it grants, over goods of one purpose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrivilegeDef {
    pub grant: Grant,
    pub purpose: crate::params::GoodUse,
}

impl PrivilegeDef {
    /// What it grants, in words after "an organization alone": "may offer tools at doors here",
    /// "may sell here tools fetched from another settlement".
    pub fn words(&self) -> String {
        match self.grant {
            Grant::Licence => format!("may offer {} at doors here", self.purpose.plural_words()),
            Grant::Staple => format!(
                "may sell here {} fetched from another settlement",
                self.purpose.plural_words()
            ),
        }
    }
}

/// An office's tenure (M7a slice BN, ADR-0024 §2): a term of days, whether the one who held it
/// may be named or drawn again for the next, and whether it is filled by lot from those who put
/// their names in rather than named by the body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tenure {
    pub term_days: u32,
    pub renewable: bool,
    pub by_lot: bool,
}

/// An office law's term as it was filled (M7a slice BN): the day it ends, and how many put their
/// names in when it was drawn by lot (0 when the body named its holder).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Term {
    pub until: i64,
    pub pool: u32,
}

/// What a law raising a building raises (M8d slice DC, ADR-0031 §6): its program, by index in
/// the catalog's buildings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RaiseDef {
    pub program: usize,
}

/// What a rule for building requires of a building begun while it is in force (M6b slice BH;
/// ADR-0022 §12). Each template names either a property or a technique (research 11-09 §5.2),
/// and applies only where its predicate holds (11-09 §1.7).
#[derive(Clone, Debug, PartialEq)]
pub struct RuleDef {
    /// What it requires.
    pub template: RuleTemplate,
    /// The technique a sponsor must know to put it forward (11-09 §1.4: remedies come from known
    /// techniques), by index in the catalog's techniques.
    pub needs: Option<usize>,
}

/// A rule's template (M6b slice BH): codes are part of content, append only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RuleTemplate {
    /// A building whose plot lies within `within_m` metres of another building's must have a
    /// roof that does not burn (a property: 11-09 §5.2), as fired tile is.
    RoofsUnburnt { within_m: f32 },
    /// A building's walls must be of one of the kinds named, a bit for each wall kind (0 wattle
    /// and daub, 1 rubble, 2 mud brick, 3 fired brick: ADR-0022 §10): a technique (11-09 §5.2).
    WallsOfKinds { kinds: u8 },
    /// A building's plot must keep at least `gap_m` metres from every other's (separation:
    /// 11-09 §5.2).
    LeastGap { gap_m: f32 },
    /// The home of a household that works fire (holds an oven or a kiln) must stand `from_m`
    /// metres from every other household's home (M8c slice CW, ADR-0030 §6: `fire.rs`'s hazard;
    /// a nuisance rule, separation by use).
    FireFromHomes { from_m: f32 },
    /// A new home, where its household's midden lies, must stand `from_m` metres from every open
    /// well (M8c slice CW, ADR-0030 §6: `contagion.rs`'s seepage).
    MiddenFromWells { from_m: f32 },
    /// A building whose walls are of one of the kinds named (a bit for each, as
    /// [`RuleTemplate::WallsOfKinds`]) may rise at most `storeys` storeys (M8d slice DB, ADR-0031
    /// §5: a dimension rule).
    MostStoreys { storeys: u8, kinds: u8 },
    /// Mass walls of more than a storey must be tied at each floor (M8d slice DB, ADR-0031 §5: a
    /// technique rule).
    TiedWalls,
}

/// The names of the wall kinds, by kind code (ADR-0022 §10).
pub const WALL_KINDS: [&str; 4] = ["wattle", "rubble", "mud_brick", "fired_brick"];

impl RuleTemplate {
    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            RuleTemplate::RoofsUnburnt { .. } => "roofs_unburnt",
            RuleTemplate::WallsOfKinds { .. } => "walls_of_kinds",
            RuleTemplate::LeastGap { .. } => "least_gap",
            RuleTemplate::FireFromHomes { .. } => "fire_from_homes",
            RuleTemplate::MiddenFromWells { .. } => "midden_from_wells",
            RuleTemplate::MostStoreys { .. } => "most_storeys",
            RuleTemplate::TiedWalls => "tied_walls",
        }
    }

    /// What it requires, in words: "a new roof within 4 m of another building must not burn".
    pub fn words(self) -> String {
        match self {
            RuleTemplate::RoofsUnburnt { within_m } => {
                format!("a new roof within {within_m:.0} m of another building must not burn")
            }
            RuleTemplate::WallsOfKinds { kinds } => {
                format!("new walls must be of {}", wall_kinds_words(kinds))
            }
            RuleTemplate::LeastGap { gap_m } => {
                format!("a new building must stand {gap_m:.0} m from every other")
            }
            RuleTemplate::FireFromHomes { from_m } => format!(
                "a new home where fire is worked must stand {from_m:.0} m from others' homes"
            ),
            RuleTemplate::MiddenFromWells { from_m } => {
                format!("a new home and its midden must stand {from_m:.0} m from every open well")
            }
            RuleTemplate::MostStoreys { storeys, kinds } => format!(
                "a new building with walls of {} must rise no more than {storeys} {}",
                wall_kinds_words(kinds),
                if storeys == 1 { "storey" } else { "storeys" }
            ),
            RuleTemplate::TiedWalls => {
                "new walls of stone or brick above a storey must be tied at each floor".to_owned()
            }
        }
    }
}

/// The wall kinds of `kinds`, a bit for each, in words: "rubble, mud brick or fired brick".
fn wall_kinds_words(kinds: u8) -> String {
    let names: Vec<&str> = ["wattle and daub", "rubble", "mud brick", "fired brick"]
        .into_iter()
        .enumerate()
        .filter(|(k, _)| kinds & (1 << k) != 0)
        .map(|(_, n)| n)
        .collect();
    match names.as_slice() {
        [] => "nothing".to_owned(),
        [one] => (*one).to_owned(),
        [rest @ .., last] => format!("{} or {last}", rest.join(", ")),
    }
}

/// Hours of the day a curfew runs, `(from, to)`: from the start of hour `from` to the start of
/// hour `to`, past midnight when `to` is not after `from`. `(0, 0)` for a law that sets none.
pub type Hours = (u8, u8);

/// Whether minute `minute_of_day` (0 to 1439) falls within `hours`.
pub fn within_hours(hours: Hours, minute_of_day: i64) -> bool {
    let (from, to) = (i64::from(hours.0) * 60, i64::from(hours.1) * 60);
    if from == to {
        return false;
    }
    let m = minute_of_day.rem_euclid(24 * 60);
    if from < to {
        (from..to).contains(&m)
    } else {
        m >= from || m < to
    }
}

/// How long `hours` run, hours.
pub fn hours_long(hours: Hours) -> u32 {
    let (from, to) = (u32::from(hours.0), u32::from(hours.1));
    if from == to { 0 } else { (to + 24 - from) % 24 }
}

/// How a polity's members meet, decide, forecast and comply (the people profile's `[polity]`
/// table; content API 31). Tuning values unless the content says otherwise.
#[derive(Clone, Debug, PartialEq)]
pub struct PolityParams {
    /// Days between a polity's routine reviews, when those who may propose look at its issues
    /// (research 09-01 §3.3: 7-30 days).
    pub review_days: u32,
    /// Days from a proposal to the gathering that decides it: time for word to go round.
    pub notice_days: u32,
    /// Minutes a gathering sits, from the start of the evening.
    pub gathering_minutes: u32,
    /// The founding custom: the share of the settlement's adults who must come for a gathering
    /// to decide anything (ADR-0013 §1).
    pub quorum_share: f64,
    /// Points per unit of a forecast gain (a change in the log of a year's food above
    /// subsistence).
    pub w_gain: f64,
    /// Points for a member's full regard for a law's sponsor.
    pub w_regard: f64,
    /// Points either way within which a member abstains.
    pub stance_margin: f64,
    /// Points for attending a gathering, before the stake.
    pub attend_base: f64,
    /// Points per point at stake in what a gathering decides.
    pub w_attend: f64,
    /// Points a gathering held in the open is worth the less on a wet day (M8d slice DC): rain
    /// keeps people from a gathering a hall would shelter.
    pub wet_attend: f64,
    /// Weight of those who regard a sponsor in what the sponsor forecasts, against their own
    /// household's 1.
    pub w_followers: f64,
    /// Points a proposal costs its sponsor: the outside option is worth this much more.
    pub propose_cost: f64,
    /// Temperature of the choice among moves, points.
    pub temperature: f64,
    /// Days a member remembers how a gathering they came to stood on a proposal: while they do,
    /// they expect no more support for the same proposal than they saw it get (research 09-05
    /// §1.2: a sponsor weighs expected success).
    pub vote_memory_days: u32,
    /// The lean years a settlement is believed to have before it has seen any, out of
    /// `prior_years`.
    pub prior_lean: f64,
    /// Years of that prior belief.
    pub prior_years: f64,
    /// A lean year's harvest, as a share of an ordinary one.
    pub lean_harvest: f64,
    /// The share of what outsiders take at places its people work that a household believes a
    /// claim on those places keeps for it (M5c slice AT, ADR-0020 §5): a design prior, since a
    /// claim is only words until outsiders heed it.
    pub claim_keeps: f64,
    /// The share of a year's food below which a household cannot live: food there is worth the
    /// most, and a household that would fall below it cannot pay a levy.
    pub subsistence_share: f64,
    /// Points for paying what a law asks, before its cost and apart from the norms each person
    /// holds (content API 42: the custom that the gathering binds is a norm, `core:norm/
    /// gathering_binds`, held by each person their own way; before it, this was 1.5 for all).
    pub comply_base: f64,
    /// Points for paying per unit of the stance a person took on the law (1 for, −1 against, 0
    /// for abstaining or absent).
    pub w_stance: f64,
    /// Days after the custom is taken from the gathering in which its new body's members weigh
    /// ending each law the old custom made (M4c slice AI, step two; research 09-11 §1.6). What
    /// they leave stands.
    pub founding_days: u32,
    /// Points a member's stance gains for each day of their household's food that gifts from the
    /// law's sponsor, given before a gathering, brought it within [`Self::owed_memory_days`] (M7c
    /// slice BY, ADR-0026 §4: what a client owes; a prior).
    pub w_owed: f64,
    /// Days a patron's gift is owed for.
    pub owed_memory_days: u32,
    /// The most days of a client household's food a patron gives it before one gathering.
    pub patron_gift_days: f64,
}

impl PolityParams {
    /// The core content's values (`content/core/people/early_farmers.toml`), for tests.
    pub fn core() -> PolityParams {
        PolityParams {
            review_days: 7,
            notice_days: 1,
            gathering_minutes: 120,
            quorum_share: 0.25,
            w_gain: 10.0,
            w_regard: 1.0,
            stance_margin: 0.25,
            attend_base: 1.0,
            w_attend: 0.5,
            wet_attend: 1.0,
            w_followers: 0.5,
            propose_cost: 0.5,
            temperature: 0.5,
            vote_memory_days: 365,
            prior_lean: 1.0,
            prior_years: 4.0,
            lean_harvest: 0.5,
            claim_keeps: 0.5,
            subsistence_share: 0.5,
            comply_base: 0.0,
            w_stance: 1.0,
            founding_days: 90,
            w_owed: 0.1,
            owed_memory_days: 365,
            patron_gift_days: 10.0,
        }
    }
}

/// Who a body's members are (ADR-0013 §2). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Membership {
    /// Every adult who lives in the settlement.
    Adults,
    /// The elder of each household: its eldest of an age to keep one (M4c slice AF; research
    /// 09-05 §1.4: aggregation by household).
    Elders,
    /// The adults of households that hold land (M4c slice AF): under a regime where the
    /// settlement holds the fields, nobody.
    Landholders,
    /// Those who keep the watch by a law in force (M4c slice AI, step three): the body a coup
    /// among them makes. When none keeps it, nobody.
    Watch,
    /// Those who hold a seat in its council (M7a slice BO, ADR-0024 §1): the body's `seats` say
    /// how many seats there are and how they are filled.
    Seats,
    /// The adults of households that belong to an organization by stake the polity recognizes
    /// (M7c slice BZ, ADR-0026 §5: closure). When it recognizes none, nobody.
    Organized,
    /// The adults of households that belong to an organization a privilege of the polity in force
    /// is granted to (M7c slice BZ). When none holds one, nobody.
    Privileged,
}

impl Membership {
    /// Every membership, in code order.
    pub const ALL: [Membership; 7] = [
        Membership::Adults,
        Membership::Elders,
        Membership::Landholders,
        Membership::Watch,
        Membership::Seats,
        Membership::Organized,
        Membership::Privileged,
    ];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The membership numbered `code`.
    pub fn from_code(code: u8) -> Option<Membership> {
        Membership::ALL.get(usize::from(code)).copied()
    }

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            Membership::Adults => "adults",
            Membership::Elders => "elders",
            Membership::Landholders => "landholders",
            Membership::Watch => "watch",
            Membership::Seats => "seats",
            Membership::Organized => "organized",
            Membership::Privileged => "privileged",
        }
    }

    /// The membership with an authored name.
    pub fn from_name(name: &str) -> Option<Membership> {
        Membership::ALL.into_iter().find(|m| m.name() == name)
    }

    /// Its members, in a count: "24 adults", "5 watchers".
    pub fn noun(self) -> &'static str {
        match self {
            Membership::Adults => "adults",
            Membership::Elders => "elders",
            Membership::Landholders => "landholders",
            Membership::Watch => "watchers",
            Membership::Seats => "councillors",
            Membership::Organized => "members of recognized organizations",
            Membership::Privileged => "members of privileged organizations",
        }
    }
}

/// How a body decides (ADR-0013 §2; research 09-05 §1.4). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassRule {
    /// Acclamation: more of those present for than against; a tie fails.
    MoreForThanAgainst,
    /// Two of every three who take a side must be for it; a tie fails (M4c slice AF; research
    /// 09-05 §1.4: a supermajority threshold).
    TwoThirds,
}

impl PassRule {
    /// Every rule, in code order.
    pub const ALL: [PassRule; 2] = [PassRule::MoreForThanAgainst, PassRule::TwoThirds];

    /// Its number in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The rule numbered `code`.
    pub fn from_code(code: u8) -> Option<PassRule> {
        PassRule::ALL.get(usize::from(code)).copied()
    }

    /// The authored name.
    pub fn name(self) -> &'static str {
        match self {
            PassRule::MoreForThanAgainst => "more_for",
            PassRule::TwoThirds => "two_thirds",
        }
    }

    /// The rule with an authored name.
    pub fn from_name(name: &str) -> Option<PassRule> {
        PassRule::ALL.into_iter().find(|r| r.name() == name)
    }
}

/// A deciding body, as values: who belongs, how many must come, and how it decides. Saved with
/// the polity, so an amendment (later) changes the polity's own body, never the content.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    /// Who belongs.
    pub members: Membership,
    /// The share of members who must be present.
    pub quorum_share: f32,
    /// How those present decide.
    pub pass: PassRule,
    /// For a council (`members` is `Seats`): its seats and how they are filled (M7a slice BO,
    /// ADR-0024 §1). `None` for a body of all its members.
    pub seats: Option<crate::seats::Seats>,
}

impl Body {
    /// The founding custom every world starts from (ADR-0013 §1): a gathering of the adults,
    /// deciding by acclamation.
    pub fn gathering(params: &PolityParams) -> Body {
        Body {
            members: Membership::Adults,
            quorum_share: params.quorum_share as f32,
            pass: PassRule::MoreForThanAgainst,
            seats: None,
        }
    }

    /// How many make up the body for its quorum, `members` of it there now: for a council, its
    /// seats, filled or empty (ADR-0024 §1: a body short of quorum fails, recorded); else its
    /// members.
    pub fn eligible(&self, members: usize) -> u32 {
        self.seats.map_or(members as u32, |s| u32::from(s.n))
    }

    /// How many changes body `self` is from `current` (research 09-05 §3.9: reform changes a
    /// gate within the procedure): who belongs, the quorum and the rule, each one; a council's
    /// seats, method and term each one more. A gathering becoming a council of its own members,
    /// or a council its electorate's gathering, is one change.
    pub fn changes_from(&self, current: &Body) -> u32 {
        let who = |b: &Body| b.seats.map_or(b.members, |s| s.electorate);
        // Taking seats or giving them up is the one change: a council's quorum is a share of its
        // seats, a gathering's of its members, so the two shares are not compared (M7a slice BQ).
        let seating = self.seats.is_some() != current.seats.is_some();
        let mut n = u32::from(!seating && (self.quorum_share - current.quorum_share).abs() > 1e-6)
            + u32::from(self.pass != current.pass)
            + u32::from(who(self) != who(current));
        match (self.seats, current.seats) {
            (Some(a), Some(b)) => {
                n += u32::from(a.n != b.n)
                    + u32::from(a.method != b.method)
                    + u32::from(a.term_days != b.term_days)
                    + u32::from(a.by_settlement != b.by_settlement);
            }
            (None, None) => {}
            _ => n += 1,
        }
        n
    }

    /// The body in words: "The adults who come to the hearth decide by acclamation: a quarter of
    /// them must come, more for than against carries it, and a tie fails."
    pub fn words(&self) -> String {
        let who = match self.members {
            Membership::Adults => "The adults who come to the hearth",
            Membership::Elders => "The elders of its households who come to the hearth",
            Membership::Landholders => {
                "The adults of households that hold land who come to the hearth"
            }
            Membership::Watch => "Those who keep the watch",
            Membership::Seats => "Those who sit in its council",
            Membership::Organized => {
                "The adults of households in an organization it recognizes who come to the hearth"
            }
            Membership::Privileged => {
                "The adults of households in an organization it grants a privilege who come to the \
                 hearth"
            }
        };
        let how = match self.pass {
            PassRule::MoreForThanAgainst => {
                "decide by acclamation: more for than against carries it, and a tie fails"
            }
            PassRule::TwoThirds => {
                "decide by acclamation: two of every three who take a side must be for it"
            }
        };
        let share = f64::from(self.quorum_share);
        let quorum = match (share * 100.0).round() as i64 {
            0 => "anyone who comes may decide".to_owned(),
            25 => "a quarter of them must come".to_owned(),
            33 => "a third of them must come".to_owned(),
            50 => "half of them must come".to_owned(),
            pct => format!("{pct} in a hundred of them must come"),
        };
        match self.seats {
            Some(s) => {
                let mut c = s.words().chars().collect::<Vec<char>>();
                if let Some(f) = c.first_mut() {
                    *f = f.to_ascii_uppercase();
                }
                format!(
                    "{}. {who} {how}; {quorum}.",
                    c.into_iter().collect::<String>()
                )
            }
            None => format!("{who} {how}; {quorum}."),
        }
    }

    /// The body as a clause to follow "so that" or a colon: "the elders of its households who
    /// come to the hearth decide by acclamation: ...; half of them must come".
    pub fn clause(&self) -> String {
        let w = self.words();
        let w = w.trim_end_matches('.');
        let mut c = w.chars();
        match c.next() {
            Some(f) => f.to_lowercase().chain(c).collect(),
            None => String::new(),
        }
    }

    /// How many must come of `eligible` members: at least one.
    pub fn quorum(&self, eligible: u32) -> u32 {
        ((f64::from(self.quorum_share) * f64::from(eligible)).ceil() as u32).max(1)
    }

    /// What the body decides with `present` of `eligible` members there, `support` for and
    /// `oppose` against. A failure is recorded as what it is, never repaired (ADR-0013 §2).
    pub fn decide(&self, eligible: u32, present: u32, support: u32, oppose: u32) -> Outcome {
        if present < self.quorum(eligible) {
            return Outcome::NoQuorum;
        }
        if support == oppose {
            return Outcome::Tied;
        }
        let passes = match self.pass {
            PassRule::MoreForThanAgainst => support > oppose,
            PassRule::TwoThirds => 3 * support >= 2 * (support + oppose),
        };
        if passes {
            Outcome::Passed
        } else {
            Outcome::Failed
        }
    }
}

/// Where a law stands. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LawStatus {
    /// Before the gathering that decides it.
    Proposed,
    /// Decided and in force.
    InForce,
    /// Not passed.
    Rejected,
    /// In force until the one it named died or left.
    Lapsed,
    /// Replaced by a later law: an amendment of the custom by a later one (M4c slice AF), or a
    /// law by one that ends or changes it (M4c slice AH).
    Superseded,
    /// A repeal that passed (M4c slice AI, step two): the law it named is superseded, and the
    /// repeal itself asks and gives nothing.
    Carried,
    /// A forum with review found it breaches a limit in force, on a challenge (M7b slice BT,
    /// ADR-0025 §4): it no longer binds. The case that struck it names it.
    StruckDown,
}

/// How a gathering's decision went. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// More were for than against.
    Passed,
    /// More were against.
    Failed,
    /// As many for as against.
    Tied,
    /// Too few came.
    NoQuorum,
}

impl Outcome {
    /// In words, after "the gathering": "passed it", "turned it down".
    pub fn words(self) -> &'static str {
        match self {
            Outcome::Passed => "passed it",
            Outcome::Failed => "turned it down",
            Outcome::Tied => "was evenly split, so it failed",
            Outcome::NoQuorum => "was too thin to decide",
        }
    }
}

/// A member's stance on a proposal (research 09-05 §1.4). Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stance {
    /// For it.
    Support,
    /// Against it.
    Oppose,
    /// Neither.
    Abstain,
}

impl Stance {
    /// +1 for, −1 against, 0 otherwise.
    pub fn sign(self) -> f64 {
        match self {
            Stance::Support => 1.0,
            Stance::Oppose => -1.0,
            Stance::Abstain => 0.0,
        }
    }
}

/// One member's stance at the gathering, with what moved it (ADR-0013 §3, stage 2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StanceRecord {
    /// Who.
    pub person: PermanentId,
    /// Their household.
    pub household: PermanentId,
    /// Where they stood.
    pub stance: Stance,
    /// Their household's forecast, points ([`PolityParams::w_gain`] times the gain).
    pub gain: f32,
    /// What their regard for the sponsor added, points.
    pub regard: f32,
    /// What talk at the hearth had moved them from their household's own lot, points (M4c slice
    /// AG; 0 before).
    pub opinion: f32,
    /// What the law does to what they hold dear, points (M4c slice AG step three; 0 before).
    pub values: f32,
    /// For an agreement, what they believe of the other polity added, points ([`PolityParams::
    /// w_regard`] times their view's warmth; M5c slice AU step three): 0 for any other law, and
    /// before.
    pub view: f32,
    /// What the household owed the law's sponsor or the one it names, for gifts before a
    /// gathering, added, points ([`PolityParams::w_owed`] times the days of its food they brought
    /// it; M7c slice BY): 0 before.
    pub owed: f32,
}

/// A gift of food a sponsor's household gave a member's before the gathering on the sponsor's law
/// (M7c slice BY, ADR-0026 §4: an ordinary gift, with its political reason recorded).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PatronGift {
    /// The sponsor who gave it.
    pub patron: PermanentId,
    /// Their household, which gave it.
    pub giver: PermanentId,
    /// The household given it.
    pub client: PermanentId,
    /// The law whose gathering it was given before.
    pub law: PermanentId,
    /// The day it was given.
    pub day: i64,
    /// Its food, kcal.
    pub kcal: f32,
    /// Days of the client household's food it was.
    pub days: f32,
}

/// What became of what a law asks of people and gives them (ADR-0013 §3, stages 6-7).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Compliance {
    /// Levies paid.
    pub complied: u32,
    /// Levies a household could not pay without falling below subsistence.
    pub could_not: u32,
    /// Levies refused.
    pub evaded: u32,
    /// Grain threshed by someone who did not know the law.
    pub unaware: u32,
    /// Grain paid in, kilograms.
    pub levied_kg: f64,
    /// Grain owed and kept back (could not, evaded and refused), kilograms.
    pub withheld_kg: f64,
    /// Levies kept back openly in a faction's refusal (M4c slice AH), and the grain, kilograms
    /// (counted in `withheld_kg` too).
    pub refused: u32,
    pub refused_kg: f64,
    /// Asks the store answered.
    pub relieved: u32,
    /// Food the store gave, kilograms.
    pub relief_kg: f64,
    /// Asks the store could not answer: it held nothing to give.
    pub unanswered: u32,
    /// For a prohibition: times someone who knew of it did what it forbids (M4b slice AD).
    pub broken: u32,
    /// Times someone who did not know of it did what it forbids.
    pub broken_unaware: u32,
    /// For a store kept by a keeper (M7c slice BY): times the keeper drew from it for their own
    /// household, and the food, kilograms (on the store's law).
    pub diverted: u32,
    pub diverted_kg: f64,
    /// Asks the store answered while low, ordered by the keeper's regard, and asks it passed
    /// over (M7c slice BY: relief by favour).
    pub favoured: u32,
    pub passed_over: u32,
    /// For a duty with a collector (M7c slice BY): purchases they assessed in full; payments
    /// they asked to assess less, those paid, and the food paid them, kilograms.
    pub assessed: u32,
    pub asked: u32,
    pub bribed: u32,
    pub bribe_kg: f64,
}

/// What a watch has done (M4b slice AC): what anyone could see of it, and where its next round
/// begins. Its hours on rounds are the polity's capacity to guard, a reported measure that nothing
/// reads (12-04 §2.4).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WatchRecord {
    /// Rounds walked.
    pub rounds: u32,
    /// Minutes on rounds, walking and standing watch.
    pub minutes: f64,
    /// Cases the watch brought before the gathering.
    pub cases: u32,
    /// The night (the day index of its evening) the latest round was walked.
    pub night: i64,
    /// Rounds walked that night.
    pub tonight: u8,
    /// Where the next round begins: an index into the settlement's households in id order.
    pub next: u32,
}

/// A law and its whole history (ADR-0013 §3): kept for ever, never pruned.
#[derive(Clone, Debug, PartialEq)]
pub struct Law {
    /// Permanent id.
    pub id: PermanentId,
    /// Its template, by index in the catalog's policies.
    pub policy: u16,
    /// What its template does: set from the catalog when it is proposed or loaded, not saved.
    pub kind: PolicyKind,
    /// The levy share proposed (0 for a law that levies nothing).
    pub levy_share: f32,
    /// The one it names, for a law that names someone (who keeps the store).
    pub holder: Option<PermanentId>,
    /// The most one ask brings, days of the asking household's food.
    pub relief_days: f32,
    /// For a law against taking: what it adds to giving back what was taken.
    pub sanction: Sanction,
    /// For a curfew: the hours it runs (`(0, 0)` for other laws).
    pub hours: Hours,
    /// Where it stands.
    pub status: LawStatus,
    /// Stage 1: who proposed it, when, and the issue it answered.
    pub sponsor: PermanentId,
    /// When it was proposed.
    pub proposed: SimTime,
    /// The issue it answered.
    pub issue: IssueKind,
    /// The day the gathering meets on it.
    pub meets_day: i64,
    /// Stage 3: when it was decided, and how.
    pub decided: Option<SimTime>,
    /// How the gathering went.
    pub outcome: Option<Outcome>,
    /// Members at the decision.
    pub eligible: u32,
    /// Stage 2: the stances of those present, in id order.
    pub stances: Vec<StanceRecord>,
    /// Stage 4: who knows it and the day each learnt it, by person id.
    pub known: Vec<(PermanentId, i64)>,
    /// Stages 6-7: what people did, and what moved.
    pub compliance: Compliance,
    /// For a watch: what it has done (M4b slice AC).
    pub watch: WatchRecord,
    /// For an amendment of the custom: the body it would make (M4c slice AF).
    pub body: Option<Body>,
    /// The law in force it would replace, if any (M4c slice AH): a common store at another
    /// share, or at none, in place of the one in force.
    pub ends: Option<PermanentId>,
    /// For the polity's side of an agreement: which (M5c slice AU).
    pub agreement: Option<PermanentId>,
    /// For a band sent by law: the settlement it goes against (M6c slice BJ).
    pub target: Option<PermanentId>,
}

impl Law {
    /// Whether `person` knows the law.
    pub fn knows(&self, person: PermanentId) -> bool {
        self.known.binary_search_by_key(&person, |k| k.0).is_ok()
    }

    /// `person` learns the law on `day`, if they did not know it. Returns whether they learnt it.
    pub fn learn(&mut self, person: PermanentId, day: i64) -> bool {
        match self.known.binary_search_by_key(&person, |k| k.0) {
            Ok(_) => false,
            Err(at) => {
                self.known.insert(at, (person, day));
                true
            }
        }
    }

    /// Those present, those for and those against.
    pub fn counts(&self) -> (u32, u32, u32) {
        let n = |s: Stance| self.stances.iter().filter(|r| r.stance == s).count() as u32;
        (
            self.stances.len() as u32,
            n(Stance::Support),
            n(Stance::Oppose),
        )
    }
}

/// A gathering called on a proposal or a case: the law before it, the day it meets, what each
/// household made of the law when word went round, the cases it is to hear, and who came.
#[derive(Clone, Debug, PartialEq)]
pub struct Gathering {
    /// The law before it, if it was called on one.
    pub law: Option<PermanentId>,
    /// The day it meets.
    pub day: i64,
    /// Each household's forecast of the law when it was proposed, points
    /// ([`PolityParams::w_gain`] times [`store_gain`]), by household id.
    pub stakes: Vec<(PermanentId, f32)>,
    /// Those who came, in id order.
    pub present: Vec<PermanentId>,
    /// The cases brought before it, by number, in the order brought (M4b slice AB).
    pub cases: Vec<u32>,
}

impl Gathering {
    /// What household `household` made of the law, points; 0 for one that formed since.
    pub fn stake(&self, household: PermanentId) -> f64 {
        self.stakes
            .binary_search_by_key(&household, |s| s.0)
            .map_or(0.0, |i| f64::from(self.stakes[i].1))
    }
}

/// Why a member stood where they did, in words: "their household stands to gain, and they think
/// well of the sponsor"; for `sponsor`, "they proposed it". `margin` is the stance margin, points.
pub fn stance_words(r: &StanceRecord, margin: f64, sponsor: PermanentId) -> String {
    if r.person == sponsor {
        return "they proposed it".to_owned();
    }
    let (gain, regard) = (f64::from(r.gain), f64::from(r.regard));
    let (talk, values) = (f64::from(r.opinion), f64::from(r.values));
    let held = if values > margin {
        "; what they hold dear drew them toward it"
    } else if values < -margin {
        "; what they hold dear turned them against it"
    } else {
        ""
    };
    let talked = if talk > margin {
        "; talk at the hearth had drawn them toward it"
    } else if talk < -margin {
        "; talk at the hearth had turned them against it"
    } else {
        ""
    };
    let household = if gain > margin {
        "their household stands to gain"
    } else if gain < -margin {
        "their household stands to lose"
    } else {
        "it makes little difference to their household"
    };
    let why = match r.stance {
        Stance::Support if gain <= margin && regard > 0.0 => {
            format!("{household}, but they think well of the sponsor")
        }
        Stance::Support if regard > 0.0 => {
            format!("{household}, and they think well of the sponsor")
        }
        Stance::Oppose | Stance::Abstain if regard > 0.0 && gain < -margin => {
            format!("{household}, though they think well of the sponsor")
        }
        Stance::Oppose | Stance::Abstain if regard < 0.0 => {
            format!("{household}, and they think better of the one it would replace")
        }
        _ => household.to_owned(),
    };
    let owed = if f64::from(r.owed) > 0.0 {
        "; their household owed the sponsor for gifts before the gathering"
    } else {
        ""
    };
    format!("{why}{held}{talked}{owed}")
}

/// What a law is, in words, to follow "proposed", "agreed to" or "turned down": "a common store,
/// taking a tenth of each harvest", "Ada as keeper of the common store". `name_of` names people.
pub fn law_words(
    law: &Law,
    policies: &[PolicyDef],
    name_of: &dyn Fn(PermanentId) -> String,
) -> String {
    let def = policies.get(usize::from(law.policy));
    if let Some(body) = law
        .body
        .filter(|_| def.is_some_and(|d| d.kind == PolicyKind::AmendBody))
    {
        return format!("a change of the custom: {}", body.clause());
    }
    // A repeal names the law it ends; without that law at hand (see [`Polity::words_of`]), only
    // that it ends one.
    if def.is_some_and(|d| d.kind == PolicyKind::Repeal) {
        return "an end to a law in force".to_owned();
    }
    if def.is_some_and(|d| d.kind == PolicyKind::ClaimPlace) {
        return "a claim on the places where people of another settlement worked: they may use \
                them only by leave"
            .to_owned();
    }
    if def.is_some_and(|d| d.kind == PolicyKind::Agreement) {
        return "an agreement with a neighbouring polity".to_owned();
    }
    // A duty or an embargo (M7c slice BV): what it is laid on, and on whom.
    if let Some(d) = def
        .filter(|d| d.kind == PolicyKind::Duty)
        .and_then(|d| d.duty.as_ref())
    {
        return format!(
            "a duty of {} of {}, owed to the common store",
            share_text(f64::from(law.levy_share)),
            d.words()
        );
    }
    // Without the settlement at hand (see [`Polity::words_of`]), only what it forbids.
    if def.is_some_and(|d| d.kind == PolicyKind::Embargo) {
        return "an embargo: nobody of theirs may buy at the doors of another settlement"
            .to_owned();
    }
    // A privilege (M7c slice BX): what it grants, without the organization at hand.
    if let Some(g) = def
        .filter(|d| d.kind == PolicyKind::Privilege)
        .and_then(|d| d.privilege.as_ref())
    {
        return format!("a privilege: an organization alone {}", g.words());
    }
    // Without the organization at hand (see [`Polity::words_of`]), only what it does.
    if def.is_some_and(|d| d.kind == PolicyKind::Recognize) {
        return "recognition of an organization: its members owe no duty on the goods it sells"
            .to_owned();
    }
    // A limit (M7b slice BT): what it forbids.
    if let Some(l) = def
        .filter(|d| d.kind == PolicyKind::Limit)
        .and_then(|d| d.limit.as_ref())
    {
        return format!("a limit: {}", l.words());
    }
    // A forum (M7b slice BR): who hears which cases.
    if let Some(f) = def
        .filter(|d| d.kind == PolicyKind::HearCases)
        .and_then(|d| d.forum.as_ref())
    {
        let heard = crate::forum::heard_words(f);
        return match law.holder {
            Some(h) => format!("{} to hear the cases brought under {heard}", name_of(h)),
            None => format!("{} to hear the cases brought under {heard}", f.words()),
        };
    }
    // Without the site at hand (see [`Polity::words_of`]), only what it is.
    if def.is_some_and(|d| d.kind == PolicyKind::BuildCrossing) {
        return "a crossing over a stream its people wade, each household doing its share of the \
                work"
            .to_owned();
    }
    // Without the site at hand (see [`Polity::words_of`]), only what it is.
    if def.is_some_and(|d| d.kind == PolicyKind::DigWell) {
        return "a well dug together beside the hearth, each household doing its share of the work"
            .to_owned();
    }
    // Without the site at hand (see [`Polity::words_of`]), only what it is.
    if def.is_some_and(|d| d.kind == PolicyKind::BuildBank) {
        return "a bank against floodwater built together, each household doing its share of the \
                work"
            .to_owned();
    }
    // Without the site at hand (see [`Polity::words_of`]), only what it is.
    if def.is_some_and(|d| d.kind == PolicyKind::BuildWay) {
        return "a passing place cleared together where carts wait, each household doing its share \
                of the work"
            .to_owned();
    }
    // Without the site at hand (see [`Polity::words_of`]), only what it is.
    if def.is_some_and(|d| d.kind == PolicyKind::BuildLine) {
        return "a line of track laid together from a pit to the hearth, each household doing its \
                share of the work"
            .to_owned();
    }
    // Without the site at hand (see [`Polity::words_of`]), only what it is (M8d slice DC).
    if let Some(d) = def.filter(|d| d.kind == PolicyKind::RaiseBuilding) {
        return format!(
            "{} raised together near the hearth, each household doing its share of the work",
            d.name.to_lowercase()
        );
    }
    // Without the layout at hand (see [`Polity::words_of`]), only what it is.
    if def.is_some_and(|d| d.kind == PolicyKind::Plan) {
        return "a plan of streets and parcels laid over the ground, its streets and square kept \
                open"
            .to_owned();
    }
    if def.is_some_and(|d| d.kind == PolicyKind::Annex) {
        return "taking in the settlement a band of theirs holds".to_owned();
    }
    // Without the site at hand (see [`Polity::words_of`]), only what it is.
    if def.is_some_and(|d| d.kind == PolicyKind::Enclose) {
        return "a palisade round the hearth, each household doing its share of the work"
            .to_owned();
    }
    if def.is_some_and(|d| d.kind == PolicyKind::KeepBank) {
        return "the keeping of a bank against floodwater, mended whenever it wants it, each \
                household doing its share of the work"
            .to_owned();
    }
    // A rule for building (M6b slice BH): what it requires of what is begun under it.
    if let Some(r) = def.and_then(|d| d.rule.as_ref()) {
        return format!(
            "a rule for building: {}, of what is begun from its day",
            r.template.words()
        );
    }
    // Without the source at hand (see [`Polity::words_of`]), only what it does.
    if def.is_some_and(|d| d.kind == PolicyKind::CloseSource) {
        return "the closing of a source of water: nobody may draw there".to_owned();
    }
    if def.is_some_and(|d| d.kind == PolicyKind::AgainstTaking) {
        return format!(
            "a law against taking: whoever is found to have taken from another household's \
             store gives back what was taken{}",
            law.sanction.words()
        );
    }
    if def.is_some_and(|d| d.kind == PolicyKind::AgainstRaiding) {
        return format!(
            "a law against raiding: whoever is found to have led a band that took from a \
             household of another settlement gives back what was taken{}",
            law.sanction.words()
        );
    }
    // Without the settlement at hand (see [`Polity::words_of`]), only who leads it.
    if def.is_some_and(|d| d.kind == PolicyKind::SendBand) {
        let who = law.holder.map_or_else(|| "someone".to_owned(), name_of);
        return format!(
            "a band led by {who} against the settlement whose people raided us, each household \
             owing one of its grown people to go"
        );
    }
    // An office's holder named in place of the one in it (M4c slice AH).
    let instead = if law.ends.is_some() {
        " in place of the one holding it"
    } else {
        ""
    };
    if def.is_some_and(|d| d.kind == PolicyKind::KeepStore) {
        return match law.holder {
            Some(h) => format!("{} as keeper of the common store{instead}", name_of(h)),
            None => "someone as keeper of the common store".to_owned(),
        };
    }
    if let Some(l) = def
        .filter(|d| d.kind == PolicyKind::Licence)
        .and_then(|d| d.licence.as_ref())
    {
        return format!(
            "that only those who know {} may be paid to tend the sick",
            l.what
        );
    }
    if let Some(k) = def
        .filter(|d| d.kind == PolicyKind::KeepApart)
        .and_then(|d| d.keep_apart)
    {
        return format!(
            "that a household with someone ill keeps to its home, and others keep from it, until \
             {} days after the last is well",
            k.days_after
        );
    }
    if let Some(m) = def
        .filter(|d| d.kind == PolicyKind::ClearMiddens)
        .and_then(|d| d.middens)
    {
        return format!(
            "that a household whose home lies within {:.0} m of an open well or a stream carries \
             its midden as far from every source",
            m.distance_m
        );
    }
    if def.is_some_and(|d| d.kind == PolicyKind::KeepRegister) {
        return match law.holder {
            Some(h) => format!("{} to keep a register of the sick{instead}", name_of(h)),
            None => "someone to keep a register of the sick".to_owned(),
        };
    }
    if def.is_some_and(|d| d.kind == PolicyKind::KeepSchool) {
        return match law.holder {
            Some(h) => format!("{} to keep the school{instead}", name_of(h)),
            None => "someone to keep the school".to_owned(),
        };
    }
    if def.is_some_and(|d| d.kind == PolicyKind::LightStreets) {
        return match law.holder {
            Some(h) => format!("{} to light the streets at night{instead}", name_of(h)),
            None => "someone to light the streets at night".to_owned(),
        };
    }
    if def.is_some_and(|d| d.kind == PolicyKind::KeepWatch) {
        return match law.holder {
            Some(h) => format!(
                "{} to keep watch over the stores at night{instead}",
                name_of(h)
            ),
            None => "someone to keep watch over the stores at night".to_owned(),
        };
    }
    if let Some(m) = def
        .filter(|d| d.kind == PolicyKind::Mint)
        .and_then(|d| d.mint)
    {
        let who = law.holder.map_or_else(|| "someone".to_owned(), name_of);
        return format!(
            "{who} to strike coin of {} g from the metal households bring, the store keeping {} \
             of what is struck{instead}",
            m.mass_g,
            share_text(f64::from(m.fee))
        );
    }
    if let Some(p) = def
        .filter(|d| d.kind == PolicyKind::PayWork)
        .and_then(|d| d.pay)
    {
        return format!(
            "pay from the store, {} an hour, for the hours households give to the common works",
            p.per_hour
        );
    }
    if def.is_some_and(|d| d.kind == PolicyKind::Curfew) {
        return format!(
            "a curfew: nobody may be away from home from {}:00 to {}:00, but the watch at its \
             rounds and those at a gathering",
            law.hours.0, law.hours.1
        );
    }
    if law.ends.is_some() {
        return replacing_words(def, law.levy_share);
    }
    format!(
        "{}, taking {} of each harvest",
        policy_name(def),
        share_text(f64::from(law.levy_share))
    )
}

/// What ending law `ended` ends, in words, after "an end to" (M4c slice AI, step two): "Ada's
/// keeping of the common store", "the curfew from 21:00 to 5:00". `name_of` names people.
pub fn ending_words(
    ended: &Law,
    policies: &[PolicyDef],
    name_of: &dyn Fn(PermanentId) -> String,
) -> String {
    let who = |what: &str| match ended.holder {
        Some(h) => format!("{}'s {what}", name_of(h)),
        None => format!("the {what}"),
    };
    match policies.get(usize::from(ended.policy)).map(|d| d.kind) {
        Some(PolicyKind::CommonStore) => "the common store's levy".to_owned(),
        Some(PolicyKind::KeepStore) => who("keeping of the common store"),
        Some(PolicyKind::KeepWatch) => who("watch over the stores at night"),
        Some(PolicyKind::LightStreets) => who("lighting of the streets"),
        Some(PolicyKind::KeepSchool) => who("keeping of the school"),
        Some(PolicyKind::KeepRegister) => who("keeping of the register of the sick"),
        Some(PolicyKind::ClearMiddens) => "the clearing of middens from the water".to_owned(),
        Some(PolicyKind::Licence) => "the licence to tend the sick for pay".to_owned(),
        Some(PolicyKind::KeepApart) => "the keeping apart of the sick".to_owned(),
        Some(PolicyKind::AgainstTaking) => "the law against taking".to_owned(),
        Some(PolicyKind::Curfew) => format!(
            "the curfew from {}:00 to {}:00",
            ended.hours.0, ended.hours.1
        ),
        Some(PolicyKind::AmendBody) => "a change of the custom".to_owned(),
        Some(PolicyKind::ClaimPlace) => "the claim on places outsiders worked".to_owned(),
        Some(PolicyKind::Agreement) => "the agreement with a neighbouring polity".to_owned(),
        Some(PolicyKind::BuildCrossing) => "the work on a crossing".to_owned(),
        Some(PolicyKind::CloseSource) => "the closing of a source of water".to_owned(),
        Some(PolicyKind::DigWell) => "the work on a well".to_owned(),
        Some(PolicyKind::BuildBank) => "the work on a flood bank".to_owned(),
        Some(PolicyKind::BuildWay) => "the work on a passing place".to_owned(),
        Some(PolicyKind::RaiseBuilding) => "the raising of a building together".to_owned(),
        Some(PolicyKind::BuildLine) => "the work on a line of track".to_owned(),
        Some(PolicyKind::Plan) => "the plan of streets and parcels".to_owned(),
        Some(PolicyKind::KeepBank) => "the keeping of a flood bank".to_owned(),
        Some(PolicyKind::BuildingRule) => "the rule for building".to_owned(),
        Some(PolicyKind::SendBand) => "the band sent by law".to_owned(),
        Some(PolicyKind::AgainstRaiding) => "the law against raiding".to_owned(),
        Some(PolicyKind::Enclose) => "the work on a palisade".to_owned(),
        Some(PolicyKind::Annex) => "the taking in of a held settlement".to_owned(),
        Some(PolicyKind::HearCases) => who("hearing of cases"),
        Some(PolicyKind::Limit) => "a limit on laws".to_owned(),
        Some(PolicyKind::Duty) => "the duty on goods bought elsewhere".to_owned(),
        Some(PolicyKind::Embargo) => "the embargo".to_owned(),
        Some(PolicyKind::Recognize) => "the recognition of an organization".to_owned(),
        Some(PolicyKind::Privilege) => "the privilege of an organization".to_owned(),
        Some(PolicyKind::DebtsHeard) => "the hearing of debts".to_owned(),
        Some(PolicyKind::LendFromStore) => "the lending from the store".to_owned(),
        Some(PolicyKind::Mint) => who("striking of coin"),
        Some(PolicyKind::PayWork) => "the pay for common work".to_owned(),
        Some(PolicyKind::Repeal) | None => "a law".to_owned(),
    }
}

/// A policy's name with its article: "a common store".
fn policy_name(def: Option<&PolicyDef>) -> String {
    def.map_or_else(
        || "a law".to_owned(),
        |d| {
            let name = d.name.to_lowercase();
            let article = match name.chars().next() {
                Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
                _ => "a",
            };
            format!("{article} {name}")
        },
    )
}

/// A store of template `def` that replaces the one in force (M4c slice AH: a petition's demand),
/// at `levy_share`, or at none: "an end to the common store's levy (the store gives what it
/// holds)"; "a common store, taking a twentieth of each harvest in place of the levy in force".
pub fn replacing_words(def: Option<&PolicyDef>, levy_share: f32) -> String {
    if levy_share <= 0.0 {
        return "an end to the common store's levy (the store gives what it holds)".to_owned();
    }
    format!(
        "{}, taking {} of each harvest in place of the levy in force",
        policy_name(def),
        share_text(f64::from(levy_share))
    )
}

/// A day in words: "3 May of year 2".
pub fn day_words(t: SimTime) -> String {
    let date = t.date();
    let month = civ_land::weather::MONTH_NAMES[usize::from(date.month.clamp(1, 12)) - 1];
    format!("{} {month} of year {}", date.day, date.year)
}

/// Every version of a polity's custom, oldest first, in words (ADR-0017 §1): "Since 3 May of
/// year 2, the founding custom: the adults who come to the hearth decide …", then "Since 9 June
/// of year 5, by the amendment Ada proposed: the elders of its households …". `name_of` names
/// people.
pub fn custom_history_words(
    polity: &Polity,
    name_of: &dyn Fn(PermanentId) -> String,
) -> Vec<String> {
    polity
        .versions
        .iter()
        .map(|v| {
            let since = day_words(v.since);
            if let Some(who) = v.seized_by {
                return format!(
                    "Since {since}, taken from the gathering by those who stood with {}, not by \
                     its procedure: {}.",
                    name_of(who),
                    v.body.clause()
                );
            }
            match v.law.and_then(|id| polity.laws.iter().find(|l| l.id == id)) {
                Some(l) => format!(
                    "Since {since}, by the amendment {} proposed: {}.",
                    name_of(l.sponsor),
                    v.body.clause()
                ),
                None => format!("Since {since}, the founding custom: {}.", v.body.clause()),
            }
        })
        .collect()
}

/// A polity's offices, in words: "Storekeeper: Ada, since 3 May of year 2", or once its holder is
/// gone and nobody named since, "Storekeeper: vacant since 9 June of year 5, when Ada died". A
/// watch adds what anyone could see of it: "Watch: Bram, since 3 May of year 2; 14 rounds, 30
/// hours on them, 2 cases brought". `gone` says how someone left ("died", "left the valley").
pub fn office_words(
    polity: &Polity,
    policies: &[PolicyDef],
    name_of: &dyn Fn(PermanentId) -> String,
    gone: &dyn Fn(PermanentId) -> String,
    today: i64,
) -> Vec<String> {
    let mut out = Vec::new();
    for (k, def) in policies.iter().enumerate() {
        if !matches!(
            def.kind,
            PolicyKind::KeepStore
                | PolicyKind::KeepWatch
                | PolicyKind::HearCases
                | PolicyKind::Mint
                | PolicyKind::LightStreets
                | PolicyKind::KeepSchool
                | PolicyKind::KeepRegister
        ) {
            continue;
        }
        // The last one named, still holding it or since gone.
        let last = polity.laws.iter().rfind(|l| {
            usize::from(l.policy) == k
                && l.holder.is_some()
                && matches!(l.status, LawStatus::InForce | LawStatus::Lapsed)
        });
        let Some(law) = last else {
            continue;
        };
        let Some(holder) = law.holder else {
            continue;
        };
        let since = law.decided.map(day_words).unwrap_or_default();
        let w = &law.watch;
        let record = if def.kind == PolicyKind::KeepWatch {
            let cases = match w.cases {
                1 => "1 case brought".to_owned(),
                n => format!("{n} cases brought"),
            };
            format!(
                "; {} round{}, {:.0} hours on them, {cases}",
                w.rounds,
                if w.rounds == 1 { "" } else { "s" },
                w.minutes / 60.0
            )
        } else {
            String::new()
        };
        // Held for a term (M7a slice BN): until when, and how it was filled.
        let term = polity.term_of(law.id);
        let held_for = term.map_or_else(String::new, |t| {
            let lot = match t.pool {
                0 => String::new(),
                1 => ", drawn by lot, the only one who put their name in".to_owned(),
                n => format!(", drawn by lot from {n} who put their names in"),
            };
            format!(
                ", until {}{lot}",
                day_words(SimTime::from_minutes(
                    t.until * civ_core::time::MINUTES_PER_DAY
                ))
            )
        });
        out.push(match law.status {
            LawStatus::InForce => {
                format!(
                    "{}: {}, since {since}{held_for}{record}",
                    def.name,
                    name_of(holder)
                )
            }
            _ => match term.filter(|t| t.until <= today) {
                Some(t) => format!(
                    "{}: vacant, since {}'s term ended on {}",
                    def.name,
                    name_of(holder),
                    day_words(SimTime::from_minutes(
                        t.until * civ_core::time::MINUTES_PER_DAY
                    ))
                ),
                None => format!(
                    "{}: vacant, since {} {}",
                    def.name,
                    name_of(holder),
                    gone(holder)
                ),
            },
        });
    }
    out
}

/// How the gathering decided a law, in words: "agreed: 20 for, 3 against; 23 of 24 adults came,
/// 6 needed". `None` before it has.
pub fn decision_words(law: &Law, body: &Body) -> Option<String> {
    let outcome = law.outcome?;
    let (present, support, oppose) = law.counts();
    let came = format!(
        "{present} of {} {} came, {} needed",
        law.eligible,
        body.members.noun(),
        body.quorum(law.eligible)
    );
    Some(match outcome {
        Outcome::Passed => format!("agreed: {support} for, {oppose} against; {came}"),
        Outcome::Failed => format!("turned down: {support} for, {oppose} against; {came}"),
        Outcome::Tied => {
            format!("evenly split, so it failed: {support} for, {oppose} against; {came}")
        }
        Outcome::NoQuorum => format!("too few came to decide: {came}"),
    })
}

/// A levy share in words: "a tenth".
pub fn share_text(share: f64) -> String {
    match (share * 100.0).round() as i64 {
        5 => "a twentieth".to_owned(),
        10 => "a tenth".to_owned(),
        20 => "a fifth".to_owned(),
        25 => "a quarter".to_owned(),
        33 => "a third".to_owned(),
        50 => "half".to_owned(),
        pct => format!("{pct} parts in a hundred"),
    }
}

/// A settlement's polity (ADR-0013 §1): its own identity, its body, its laws and its store.
#[derive(Clone, Debug, PartialEq)]
pub struct Polity {
    /// Permanent id, apart from the settlement's.
    pub id: PermanentId,
    /// The settlement whose residents are its members.
    pub settlement: PermanentId,
    /// When it was founded.
    pub founded: SimTime,
    /// The body that decides.
    pub body: Body,
    /// The common store, by good (ADR-0013 §4): kept at the hearth, under no roof yet.
    pub stores: Vec<f64>,
    /// When the store was last brought up to date.
    pub stores_at: SimTime,
    /// What came into and went out of the store, and what it lost.
    pub flows: Flows,
    /// Every law proposed here, oldest first.
    pub laws: Vec<Law>,
    /// The gathering called, if any.
    pub gathering: Option<Gathering>,
    /// The day of its last routine review.
    pub reviewed: i64,
    /// Every version of its custom, oldest first: the founding custom, then each amendment
    /// (ADR-0017 §1). The last is `body`.
    pub versions: Vec<CustomVersion>,
    /// The places its claims name, each with the law that names it (M5c slice AT, ADR-0020 §5):
    /// recorded when the claim is proposed, and claimed while that law is in force.
    pub claimed: Vec<(PermanentId, crate::uses::Place)>,
    /// Where its laws to build a crossing would build one, each with its law (M5c slice AW, step
    /// three): recorded when the law is proposed.
    pub sites: Vec<(PermanentId, CrossingSite)>,
    /// The sources of water its laws to close one name, each with its law (M6a slice BA, step
    /// four): recorded when the law is proposed, and closed while that law is in force.
    pub closed: Vec<(PermanentId, crate::uses::Place)>,
    /// Where its laws to dig a well together would dig one, each with its law (M6a slice BA,
    /// step four): recorded when the law is proposed.
    pub wells: Vec<(PermanentId, WellSite)>,
    /// Where each law to build a flood bank would build it, by law (M6b slice BG).
    pub banks: Vec<(PermanentId, BankSite)>,
    /// What each household reckoned, when it was proposed, a law's bank would save it a year,
    /// hours: the losses it remembered at the places the bank would keep dry (law, household,
    /// hours), by law then household.
    pub bank_saves: Vec<(PermanentId, PermanentId, f32)>,
    /// The hours each household has given to a law's bank (law, household, hours), by law then
    /// household (research 09-06: labour asked and labour given kept apart).
    pub bank_given: Vec<(PermanentId, PermanentId, f32)>,
    /// The banks its laws built whose giving way its chronicle has told, by earthwork id.
    pub banks_failed: Vec<PermanentId>,
    /// Which bank each law keeping one keeps (keep law, the law that built it), in the order made
    /// (M6b slice BG).
    pub bank_keeps: Vec<(PermanentId, PermanentId)>,
    /// Where each law to raise a palisade would raise it, by law (M6c slice BK).
    pub enclosures: Vec<(PermanentId, EnclosureSite)>,
    /// What each household reckoned, when it was proposed, a law's palisade would keep it a year,
    /// in hours of its work (law, household, hours), by law then household.
    pub enclosure_saves: Vec<(PermanentId, PermanentId, f32)>,
    /// Where each law to clear a passing place would clear it, by law (M8b slice CQ, third
    /// part).
    pub ways: Vec<(PermanentId, WaySite)>,
    /// What each household reckoned, when it was proposed, a law's passing place would spare it
    /// a year, hours of waiting (law, household, hours), by law then household.
    pub way_saves: Vec<(PermanentId, PermanentId, f32)>,
    /// The hours each household has given to a law's passing place (law, household, hours), by
    /// law then household.
    pub way_given: Vec<(PermanentId, PermanentId, f32)>,
    /// The building each law raising one raises, by law (M8d slice DC).
    pub raises: Vec<(PermanentId, RaiseSite)>,
    /// The hours each household has given to a law's building (law, household, hours), by law
    /// then household (M8d slice DC).
    pub raise_given: Vec<(PermanentId, PermanentId, f32)>,
    /// The storehouse it raised that stands finished, if any (M8d slice DC): the common store
    /// is kept under its roof. Looked over monthly.
    pub storehouse: Option<PermanentId>,
    /// The hall it raised that stands finished, if any (M8d slice DC): its gatherings meet under
    /// its roof, whatever the weather. Looked over monthly.
    pub hall: Option<PermanentId>,
    /// The watch tower it raised that stands finished, if any (M8d slice DC): its watch looks out
    /// from it. Looked over monthly.
    pub tower: Option<PermanentId>,
    /// The sickhouse it raised that stands finished, if any (M10c slice EE, fourth part): the sick
    /// of its households may lie there.
    pub sickhouse: Option<PermanentId>,
    /// The plan each law of a plan lays, by law (M8c slice CT, second part): in force, it
    /// reserves.
    pub plans: Vec<(PermanentId, PlanLaid)>,
    /// What it saw of its settlements' ground within memory (M8c slice CT, second part), oldest
    /// first: what raises a plan's issues.
    pub ground_seen: Vec<GroundSeen>,
    /// The buildings begun on ground a plan in force reserves (M8c slice CT, third part), in the
    /// order begun.
    pub encroachments: Vec<Encroachment>,
    /// The settlements it holds besides its seat (`settlement`), in the order taken (M6c slice
    /// BM, ADR-0023 §9): their residents are its members too.
    pub held: Vec<HeldSettlement>,
    /// Its deciding closed, when another polity took it in (M6c slice BM): its record is kept.
    pub ended: Option<PolityEnd>,
    /// The terms of its office laws held for a term, by law id (M7a slice BN).
    pub terms: Vec<(PermanentId, Term)>,
    /// Its council's seats held now, by seat (M7a slice BO, ADR-0024 §1): empty unless its body
    /// is a council, and while nobody sits.
    pub seated: Vec<crate::seats::Seat>,
    /// Every election it called, oldest first (ADR-0024 §3).
    pub elections: Vec<crate::seats::Election>,
    /// Gifts its sponsors gave before its gatherings, oldest first, within
    /// [`PolityParams::owed_memory_days`] (M7c slice BY).
    pub patron_gifts: Vec<PatronGift>,
    /// What its settlements measured of the losses a public building would answer, a year each,
    /// the latest [`MEASURED_YEARS`] (M8d slice DC, ADR-0031 §6).
    pub measured: Vec<Measured>,
}

/// Years of [`Measured`] a polity keeps.
pub const MEASURED_YEARS: usize = 3;

/// What a polity's settlements measured in a year of the losses a public building would answer
/// (M8d slice DC, ADR-0031 §6: public programs answer measured losses only): its common store's
/// spoilage, the gatherings it held in the open and how many of them on a wet day, and the fires
/// that reached a roof, noticed only once the covering had caught.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Measured {
    /// The year.
    pub year: i64,
    /// Food its common store lost to spoiling, kcal.
    pub spoiled_kcal: f64,
    /// Gatherings it held in the open, with no hall of its standing.
    pub gatherings: u32,
    /// Of them, those held on a wet day.
    pub wet_gatherings: u32,
    /// Buildings whose covering caught: fires not noticed before they reached a roof.
    pub roof_fires: u32,
    /// Hours those who came gave to gatherings in the open that a wet day left without a quorum
    /// (schema 166): what a hall would have spared them.
    pub wet_lost_h: f64,
    /// Hours of work putting right the buildings of fires nobody saw for a while after their
    /// covering caught, but spread from a neighbour (schema 167): what a watch tower answers.
    pub late_fire_h: f64,
    /// Days a severe case of its households lay ill with nobody tending them (schema 206, M10c
    /// slice EE, fourth part): what a sickhouse answers.
    pub untended_days: f64,
}

/// How a polity came to hold a settlement besides its seat (M6c slice BM, ADR-0023 §9). Codes are
/// part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeldHow {
    /// Its own gathering annexed it, having held it from outside.
    Annexed = 0,
    /// Both gatherings agreed to be one body (the *one body* clause).
    Merged = 1,
}

impl HeldHow {
    /// Its code in saves.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// The way with code `code`.
    pub fn from_code(code: u8) -> Option<HeldHow> {
        [HeldHow::Annexed, HeldHow::Merged]
            .get(usize::from(code))
            .copied()
    }

    /// In words, after "since": "annexed", "merged".
    pub fn words(self) -> &'static str {
        match self {
            HeldHow::Annexed => "annexed",
            HeldHow::Merged => "joined by agreement",
        }
    }
}

/// A settlement a polity holds besides its seat: since when, how, and the polity that held it
/// before.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeldSettlement {
    pub settlement: PermanentId,
    pub since: i64,
    pub how: HeldHow,
    pub from: PermanentId,
}

/// How a polity's deciding closed: the day, the polity that took it in, and how.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolityEnd {
    pub day: i64,
    pub by: PermanentId,
    pub how: HeldHow,
}

/// Where a law would raise a palisade (M6c slice BK; ADR-0023 §6): the point its ring is drawn
/// round (the hearth), its radius, its gates, how many homes it takes in, the work it would take,
/// and the enclosure once begun.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnclosureSite {
    pub centre: (f32, f32),
    pub radius_m: f32,
    pub gates: u8,
    /// Homes inside it when it was proposed.
    pub inside: u16,
    /// The work it takes, hours.
    pub labour_h: f32,
    /// Its enclosure once begun.
    pub work: Option<PermanentId>,
}

impl EnclosureSite {
    /// The length of its line, metres.
    pub fn length_m(&self) -> f64 {
        std::f64::consts::TAU * f64::from(self.radius_m)
    }
}

/// Where a law would clear a passing place (M8b slice CQ, third part; ADR-0029 §6): the cell
/// at the middle of the single-cart stretch, the hours of work clearing it would take, and, once
/// begun, the way it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaySite {
    pub cell: u32,
    pub labour_h: f64,
    pub work: Option<PermanentId>,
}

/// The building a law raises (M8d slice DC, ADR-0031 §6): its program, by index in the catalog's
/// buildings; the hours of a capable adult's work raising it takes, its materials made and
/// carried among them, as reckoned when it was proposed; and the building once laid out. With
/// the law's own day of proposal, its dates (ADR-0031 §6: proposal, start, first use and
/// completion are kept, and a stall stays visible): when it was laid out, last worked on, first
/// used and finished, and when it stalled, if it did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RaiseSite {
    pub program: u16,
    pub labour_h: f64,
    pub building: Option<PermanentId>,
    pub begun: Option<SimTime>,
    pub last_work: Option<SimTime>,
    pub first_use: Option<SimTime>,
    pub finished: Option<SimTime>,
    pub stalled: Option<SimTime>,
}

/// A plan's template (M8c slice CT, second part; ADR-0030 §2): the grid it lays, the extents in
/// blocks (along and across) a sponsor chooses among by the households it expects, smallest
/// first, the hours staking out a parcel takes, and the technique a sponsor must know to put it
/// forward.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanDef {
    pub grid: civ_land::plans::Grid,
    pub extents: Vec<(u8, u8)>,
    pub stake_h: f32,
    pub needs: Option<usize>,
    /// How its keeper gives its parcels (M8c slice CU).
    pub allot: civ_land::AllotRule,
    /// For a parcel given for a fee: the good, by index in the catalog's goods, and the amount
    /// paid the store once.
    pub fee: Option<(usize, f32)>,
    /// For a parcel given for a ground rent (M8c slice CU, third part): the food, by index in the
    /// catalog's goods, and the amount owed the store each year the parcel is held.
    pub rent: Option<(usize, f32)>,
    /// How many phases its parcels open in, outward from where it is laid, and the share of
    /// those open given before the next opens (M8c slice CU, fourth part).
    pub phases: u8,
    pub open_at: f32,
    /// What it does with the fields under the ground it reserves (M8c slice CU, fifth part), and
    /// for [`FieldClause::Paid`] the good, by index in the catalog's goods, and the amount the
    /// store offers a field's holder a hectare.
    pub fields: FieldClause,
    pub pay: Option<(usize, f32)>,
    /// Its districts (M8c slice CW, ADR-0030 §6: zoning as a clause of a plan): each phase's
    /// parcels, by phase from 0, and the uses allowed there, a bit for each by its place in
    /// [`civ_land::PlotUse::ALL`]. A phase named in none allows every use.
    pub districts: Vec<(u8, u8)>,
}

impl PlanDef {
    /// Whether a parcel of phase `phase` (from 0) allows a building of use `use_`.
    pub fn allows(&self, phase: u8, use_: civ_land::PlotUse) -> bool {
        self.districts
            .iter()
            .find(|d| d.0 == phase)
            .is_none_or(|d| d.1 & use_bit(use_) != 0)
    }
}

/// The bit for use `use_` in a district's uses: its place in [`civ_land::PlotUse::ALL`].
pub fn use_bit(use_: civ_land::PlotUse) -> u8 {
    civ_land::PlotUse::ALL
        .iter()
        .position(|&u| u == use_)
        .map_or(0, |i| 1 << i)
}

/// What a plan does with a field under the ground it reserves (ADR-0030 §4: a clause per kind of
/// claim a layout touches). Never a standing home, under any.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldClause {
    /// The layout bends round fields: none is touched.
    #[default]
    Around,
    /// Reserved without consent, its standing crop left to come in; its holder and those who
    /// work it hold it against the gathering.
    Taken,
    /// The store offers its holder the template's goods a hectare, which it takes or refuses by
    /// what breaking like ground elsewhere would cost it; refused, it stays out of the plan.
    Paid,
}

impl FieldClause {
    /// Every clause.
    pub const ALL: [FieldClause; 3] = [FieldClause::Around, FieldClause::Taken, FieldClause::Paid];

    /// Its name in content: "around", "taken", "paid".
    pub fn name(self) -> &'static str {
        match self {
            FieldClause::Around => "around",
            FieldClause::Taken => "taken",
            FieldClause::Paid => "paid",
        }
    }

    /// The clause named `name` in content.
    pub fn from_name(name: &str) -> Option<FieldClause> {
        FieldClause::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// The plan a law lays (M8c slice CT, second part; ADR-0030 §2): its version (1 for its polity's
/// first, one more for each after), the grid laid, and the layout as realised when it was
/// proposed, which does not change when the ground does.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanLaid {
    pub version: u16,
    pub grid: civ_land::plans::Grid,
    pub layout: civ_land::plans::Layout,
    /// The hours its keeper has spent staking the parcels given (M8c slice CU, first part).
    pub staked_h: f32,
    /// Fields under its ground whose holders refused what the store offered (fifth part): left
    /// out of it, and not asked again.
    pub refused_fields: Vec<PermanentId>,
}

/// What a polity saw of its settlements' ground (M8c slice CT, second part): the day, the
/// settlement, the household it was of, and the issue it raises (`NoGround`, `WayBuiltOver` or
/// `NewGround`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroundSeen {
    pub day: i64,
    pub settlement: PermanentId,
    pub household: PermanentId,
    pub issue: IssueKind,
}

/// A building begun on ground a plan in force reserves (M8c slice CT, third part): the day, the
/// plan's law, the household that began it, and the building.
#[derive(Clone, Debug, PartialEq)]
pub struct Encroachment {
    pub day: i64,
    pub law: PermanentId,
    pub household: PermanentId,
    pub building: PermanentId,
    /// Who saw it begun, in id order (M8c slice CV, ADR-0030 §5: "recorded with who saw it"):
    /// the plan's keeper living there, and the grown of the households whose homes lie within
    /// sight of it, never its own.
    pub seen_by: Vec<PermanentId>,
    /// What a finding under the plan ordered for it, if any (M8c slice CV, second part).
    pub order: Option<PlanOrder>,
}

/// What a finding under a plan ordered for a building on its reserved ground (M8c slice CV,
/// second part; ADR-0030 §5: removal, or regularisation for a yearly payment).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlanOrder {
    /// To be taken down on day `due`.
    Remove { due: i64 },
    /// Taken down on `day`.
    Removed { day: i64 },
    /// It may stand, its household owing the store `kcal` each year from day `since`.
    Regularised { since: i64, kcal: f32 },
}

impl PlanOrder {
    /// Its code in saves.
    pub fn code(self) -> u8 {
        match self {
            PlanOrder::Remove { .. } => 1,
            PlanOrder::Removed { .. } => 2,
            PlanOrder::Regularised { .. } => 3,
        }
    }

    /// From its code, day and amount in saves.
    pub fn from_code(code: u8, day: i64, kcal: f32) -> Option<PlanOrder> {
        match code {
            1 => Some(PlanOrder::Remove { due: day }),
            2 => Some(PlanOrder::Removed { day }),
            3 => Some(PlanOrder::Regularised { since: day, kcal }),
            _ => None,
        }
    }

    /// Its day and amount in saves.
    pub fn day_kcal(self) -> (i64, f32) {
        match self {
            PlanOrder::Remove { due } => (due, 0.0),
            PlanOrder::Removed { day } => (day, 0.0),
            PlanOrder::Regularised { since, kcal } => (since, kcal),
        }
    }
}

/// Where a law would build a flood bank (M6b slice BG; ADR-0022 §11): the reach whose water it
/// keeps out, its straight crest line from `a` to `b` in metres from the map's corner, its
/// section, the ground it reserves and its borrow pit's, and the work it would take.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BankSite {
    /// The river reach whose floodwater it keeps out.
    pub reach: u32,
    pub a: [f64; 2],
    pub b: [f64; 2],
    /// Its finished height above its ground, metres.
    pub height_m: f64,
    pub crest_width_m: f64,
    pub side_run: f64,
    /// Compacted volume per cubic metre dug in place.
    pub compacted_per_bank: f64,
    /// The ground its trapezoid takes.
    pub envelope: civ_land::fields::RectCm,
    /// The pit its earth is dug from.
    pub pit: civ_land::fields::RectCm,
    /// The work it takes, hours: digging, carrying and placing.
    pub labour_h: f64,
    /// The work its mending has asked of its households since it was built, hours: each is asked
    /// its share of building and mending together.
    pub upkeep_h: f64,
    /// Whether its households are asked to mend it now: from when a law keeping it finds it
    /// wanting until all its mending is done.
    pub mending: bool,
    /// Its earthwork once begun: the bank's own id (its segment's `work`).
    pub work: Option<PermanentId>,
}

impl BankSite {
    /// Its length, metres.
    pub fn length_m(&self) -> f64 {
        (self.a[0] - self.b[0]).hypot(self.a[1] - self.b[1])
    }
}

/// Where a law would dig a well (M6a slice BA, step four): its cell and ground, its well system
/// by its place in the catalog, how deep it would be dug and the work it would take.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WellSite {
    /// The cell its shaft is sunk in.
    pub cell: u32,
    /// The ground its digging would take.
    pub rect: civ_land::fields::RectCm,
    /// Its well system, by index in the catalog's.
    pub system: u16,
    /// How deep it would be dug, metres: a metre below where its people expect water.
    pub target_m: f32,
    /// The work it takes, hours.
    pub labour_h: f32,
}

/// Where a law would build a crossing (M5c slice AW, step three): the river cell, the bridge
/// system by its place in the catalog, the channel's width there, and the work it would take.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrossingSite {
    /// The river cell it spans.
    pub cell: u32,
    /// Its bridge system, by index in the catalog's.
    pub system: u16,
    /// The clear span, metres.
    pub span_m: f32,
    /// The work it takes, hours.
    pub labour_h: f32,
}

/// A version of a polity's custom (ADR-0017 §1): its body, since when, and the law that made it
/// (none for the founding custom).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CustomVersion {
    /// The body.
    pub body: Body,
    /// When it began.
    pub since: SimTime,
    /// The amendment that made it.
    pub law: Option<PermanentId>,
    /// For a custom taken from the gathering rather than amended (M4c slice AI): who called those
    /// who stood with it.
    pub seized_by: Option<PermanentId>,
}

impl Polity {
    /// The body that decided what it decided at `t`: the version of the custom in force before
    /// then (one that began at `t` was made by what was decided then, under the one before).
    pub fn body_at(&self, t: SimTime) -> &Body {
        self.versions
            .iter()
            .rev()
            .find(|v| v.since < t)
            .or_else(|| self.versions.first())
            .map_or(&self.body, |v| &v.body)
    }

    /// A new polity for `settlement` under the founding custom.
    pub fn found(
        id: PermanentId,
        settlement: PermanentId,
        now: SimTime,
        params: &PolityParams,
    ) -> Polity {
        Polity {
            id,
            settlement,
            founded: now,
            body: Body::gathering(params),
            stores: Vec::new(),
            stores_at: now,
            flows: Flows::default(),
            laws: Vec::new(),
            gathering: None,
            reviewed: now.day_index(),
            versions: vec![CustomVersion {
                body: Body::gathering(params),
                since: now,
                law: None,
                seized_by: None,
            }],
            claimed: Vec::new(),
            sites: Vec::new(),
            closed: Vec::new(),
            wells: Vec::new(),
            banks: Vec::new(),
            bank_saves: Vec::new(),
            bank_given: Vec::new(),
            banks_failed: Vec::new(),
            bank_keeps: Vec::new(),
            enclosures: Vec::new(),
            enclosure_saves: Vec::new(),
            ways: Vec::new(),
            way_saves: Vec::new(),
            way_given: Vec::new(),
            raises: Vec::new(),
            raise_given: Vec::new(),
            storehouse: None,
            hall: None,
            tower: None,
            sickhouse: None,
            plans: Vec::new(),
            ground_seen: Vec::new(),
            encroachments: Vec::new(),
            held: Vec::new(),
            ended: None,
            terms: Vec::new(),
            seated: Vec::new(),
            elections: Vec::new(),
            patron_gifts: Vec::new(),
            measured: Vec::new(),
        }
    }

    /// Year `year`'s measured losses, begun if new; the oldest beyond [`MEASURED_YEARS`] are
    /// let go (M8d slice DC).
    pub fn measured_mut(&mut self, year: i64) -> &mut Measured {
        if self.measured.last().is_none_or(|m| m.year < year) {
            self.measured.push(Measured {
                year,
                ..Measured::default()
            });
            let drop = self.measured.len().saturating_sub(MEASURED_YEARS);
            self.measured.drain(..drop);
        }
        let i = self
            .measured
            .iter()
            .rposition(|m| m.year <= year)
            .unwrap_or(0);
        &mut self.measured[i]
    }

    /// The term of office law `law`, if it is held for one (M7a slice BN).
    pub fn term_of(&self, law: PermanentId) -> Option<Term> {
        self.terms
            .binary_search_by_key(&law, |t| t.0)
            .ok()
            .map(|i| self.terms[i].1)
    }

    /// Records office law `law`'s term (M7a slice BN), keeping the terms in law order.
    pub fn set_term(&mut self, law: PermanentId, term: Term) {
        match self.terms.binary_search_by_key(&law, |t| t.0) {
            Ok(i) => self.terms[i].1 = term,
            Err(i) => self.terms.insert(i, (law, term)),
        }
    }

    /// Whether, still deciding, it holds settlement `s`: its seat or one it took in (M6c slice
    /// BM). Residence in any is membership (ADR-0023 §9).
    pub fn holds(&self, s: Option<PermanentId>) -> bool {
        self.ended.is_none()
            && s.is_some_and(|s| {
                s == self.settlement || self.held.iter().any(|h| h.settlement == s)
            })
    }

    /// Its settlements, the seat first, then those it took in, in the order taken.
    pub fn settlements(&self) -> Vec<PermanentId> {
        std::iter::once(self.settlement)
            .chain(self.held.iter().map(|h| h.settlement))
            .collect()
    }

    /// Where law `law` would raise a palisade, if it is one that would (M6c slice BK).
    pub fn enclosure_site_of(&self, law: PermanentId) -> Option<EnclosureSite> {
        self.enclosures.iter().find(|e| e.0 == law).map(|e| e.1)
    }

    /// The hours a year household `household` reckoned law `law`'s palisade would keep it, when
    /// the law was proposed (M6c slice BK).
    pub fn enclosure_saves_of(&self, law: PermanentId, household: PermanentId) -> f32 {
        self.enclosure_saves
            .binary_search_by_key(&(law, household), |s| (s.0, s.1))
            .map_or(0.0, |i| self.enclosure_saves[i].2)
    }

    /// Where law `law` would build a crossing, if it is one that would (M5c slice AW).
    pub fn site_of(&self, law: PermanentId) -> Option<CrossingSite> {
        self.sites.iter().find(|s| s.0 == law).map(|s| s.1)
    }

    /// Where law `law` would dig a well, if it is one that would (M6a slice BA).
    pub fn well_site_of(&self, law: PermanentId) -> Option<WellSite> {
        self.wells.iter().find(|w| w.0 == law).map(|w| w.1)
    }

    /// Where law `law` would build a flood bank, if it is one that would (M6b slice BG).
    pub fn bank_site_of(&self, law: PermanentId) -> Option<BankSite> {
        self.banks.iter().find(|b| b.0 == law).map(|b| b.1)
    }

    /// The bank law `law` keeps, if it is one that keeps one: the law that built it, and its
    /// site (M6b slice BG).
    pub fn kept_bank_of(&self, law: PermanentId) -> Option<(PermanentId, BankSite)> {
        let built = self.bank_keeps.iter().find(|k| k.0 == law)?.1;
        Some((built, self.bank_site_of(built)?))
    }

    /// The law in force that keeps the bank law `built` built, if any (M6b slice BG).
    pub fn keeper_of(&self, built: PermanentId) -> Option<PermanentId> {
        self.bank_keeps
            .iter()
            .filter(|k| k.1 == built)
            .map(|k| k.0)
            .find(|&k| {
                self.laws
                    .iter()
                    .any(|l| l.id == k && l.status == LawStatus::InForce)
            })
    }

    /// The hours all households have given to law `law`'s bank.
    pub fn bank_given_all(&self, law: PermanentId) -> f64 {
        self.bank_given
            .iter()
            .filter(|g| g.0 == law)
            .map(|g| f64::from(g.2))
            .sum()
    }

    /// What household `household` reckoned law `law`'s bank would save it a year, hours.
    pub fn bank_saves_of(&self, law: PermanentId, household: PermanentId) -> f32 {
        self.bank_saves
            .binary_search_by_key(&(law, household), |s| (s.0, s.1))
            .map_or(0.0, |i| self.bank_saves[i].2)
    }

    /// The hours household `household` has given to law `law`'s bank.
    pub fn bank_given_by(&self, law: PermanentId, household: PermanentId) -> f32 {
        self.bank_given
            .binary_search_by_key(&(law, household), |s| (s.0, s.1))
            .map_or(0.0, |i| self.bank_given[i].2)
    }

    /// Household `household` gives `hours` more to law `law`'s bank.
    pub fn give_to_bank(&mut self, law: PermanentId, household: PermanentId, hours: f32) {
        match self
            .bank_given
            .binary_search_by_key(&(law, household), |s| (s.0, s.1))
        {
            Ok(i) => self.bank_given[i].2 += hours,
            Err(i) => self.bank_given.insert(i, (law, household, hours)),
        }
    }

    /// Where law `law` would clear a passing place, if it is one that would (M8b slice CQ, third
    /// part).
    pub fn way_site_of(&self, law: PermanentId) -> Option<WaySite> {
        self.ways.iter().find(|w| w.0 == law).map(|w| w.1)
    }

    /// What household `household` reckoned law `law`'s passing place would spare it a year,
    /// hours.
    pub fn way_saves_of(&self, law: PermanentId, household: PermanentId) -> f32 {
        self.way_saves
            .binary_search_by_key(&(law, household), |s| (s.0, s.1))
            .map_or(0.0, |i| self.way_saves[i].2)
    }

    /// The hours household `household` has given to law `law`'s passing place.
    pub fn way_given_by(&self, law: PermanentId, household: PermanentId) -> f32 {
        self.way_given
            .binary_search_by_key(&(law, household), |s| (s.0, s.1))
            .map_or(0.0, |i| self.way_given[i].2)
    }

    /// Household `household` gives `hours` more to law `law`'s passing place.
    pub fn give_to_way(&mut self, law: PermanentId, household: PermanentId, hours: f32) {
        match self
            .way_given
            .binary_search_by_key(&(law, household), |s| (s.0, s.1))
        {
            Ok(i) => self.way_given[i].2 += hours,
            Err(i) => self.way_given.insert(i, (law, household, hours)),
        }
    }

    /// The building law `law` raises, if it raises one (M8d slice DC).
    pub fn raise_site_of(&self, law: PermanentId) -> Option<RaiseSite> {
        self.raises.iter().find(|r| r.0 == law).map(|r| r.1)
    }

    /// The hours household `household` has given to law `law`'s building (M8d slice DC).
    pub fn raise_given_by(&self, law: PermanentId, household: PermanentId) -> f32 {
        self.raise_given
            .binary_search_by_key(&(law, household), |s| (s.0, s.1))
            .map_or(0.0, |i| self.raise_given[i].2)
    }

    /// Household `household` gives `hours` more to law `law`'s building (M8d slice DC).
    pub fn give_to_raise(&mut self, law: PermanentId, household: PermanentId, hours: f32) {
        match self
            .raise_given
            .binary_search_by_key(&(law, household), |s| (s.0, s.1))
        {
            Ok(i) => self.raise_given[i].2 += hours,
            Err(i) => self.raise_given.insert(i, (law, household, hours)),
        }
    }

    /// The plan law `law` lays, if it is one (M8c slice CT, second part).
    pub fn plan_of(&self, law: PermanentId) -> Option<&PlanLaid> {
        self.plans.iter().find(|p| p.0 == law).map(|p| &p.1)
    }

    /// The plans of its laws in force, oldest first (M8c slice CT): what they reserve.
    pub fn plans_in_force(&self) -> impl Iterator<Item = (PermanentId, &PlanLaid)> {
        self.plans.iter().filter_map(|(law, p)| {
            self.laws
                .iter()
                .any(|l| l.id == *law && l.status == LawStatus::InForce)
                .then_some((*law, p))
        })
    }

    /// It saw `seen` (M8c slice CT, second part): kept once a day for each household and issue.
    pub fn see_ground(&mut self, seen: GroundSeen) {
        if !self
            .ground_seen
            .iter()
            .rev()
            .take_while(|g| g.day == seen.day)
            .any(|g| *g == seen)
        {
            self.ground_seen.push(seen);
        }
    }

    /// The source of water law `law` would close, if it is one that would (M6a slice BA).
    pub fn closure_of(&self, law: PermanentId) -> Option<crate::uses::Place> {
        self.closed.iter().find(|c| c.0 == law).map(|c| c.1)
    }

    /// The sources of water it closes now, each with the law in force that closes it (the first
    /// made), in place order (M6a slice BA, step four).
    pub fn closed_now(&self) -> Vec<(crate::uses::Place, PermanentId)> {
        let mut out: Vec<(crate::uses::Place, PermanentId)> = self
            .closed
            .iter()
            .filter(|(law, _)| {
                self.laws
                    .iter()
                    .any(|l| l.id == *law && l.status == LawStatus::InForce)
            })
            .map(|&(law, p)| (p, law))
            .collect();
        out.sort_unstable();
        out.dedup_by_key(|c| c.0);
        out
    }

    /// The law in force that claims `place`, if any (the first made; M5c slice AT).
    pub fn claim_on(&self, place: crate::uses::Place) -> Option<PermanentId> {
        self.claimed
            .iter()
            .filter(|&&(_, p)| p == place)
            .map(|&(law, _)| law)
            .find(|&law| {
                self.laws
                    .iter()
                    .any(|l| l.id == law && l.status == LawStatus::InForce)
            })
    }

    /// The places it claims now: those named by its claims in force (M5c slice AT), in place
    /// order.
    pub fn claims_now(&self) -> Vec<crate::uses::Place> {
        let mut out: Vec<crate::uses::Place> = self
            .claimed
            .iter()
            .filter(|(law, _)| {
                self.laws
                    .iter()
                    .any(|l| l.id == *law && l.status == LawStatus::InForce)
            })
            .map(|&(_, p)| p)
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// The law before the gathering, if any.
    pub fn agenda(&self) -> Option<&Law> {
        self.laws.iter().find(|l| l.status == LawStatus::Proposed)
    }

    /// The forum a *hear cases* law in force names, and that law (M7b slice BR, ADR-0025 §1):
    /// one at a time.
    pub fn forum<'a>(
        &'a self,
        policies: &'a [PolicyDef],
    ) -> Option<(&'a Law, &'a crate::forum::ForumDef)> {
        self.laws
            .iter()
            .filter(|l| l.status == LawStatus::InForce && l.kind == PolicyKind::HearCases)
            .find_map(|l| Some((l, policies.get(usize::from(l.policy))?.forum.as_ref()?)))
    }

    /// The one who keeps the register of the sick by a law in force, and that law (M10c slice EE,
    /// second part).
    pub fn registrar(&self) -> Option<(PermanentId, &Law)> {
        self.office(PolicyKind::KeepRegister)
    }

    /// The one who keeps the store under their roof by a law in force, and that law.
    pub fn keeper(&self) -> Option<(PermanentId, &Law)> {
        self.office(PolicyKind::KeepStore)
    }

    /// The one who keeps watch by a law in force, and that law (M4b slice AC): the first, when
    /// several do.
    pub fn watcher(&self) -> Option<(PermanentId, &Law)> {
        self.office(PolicyKind::KeepWatch)
    }

    /// Everyone who keeps watch by a law in force, with their laws, oldest law first (M4c slice
    /// AI: a watch of several).
    pub fn watchers(&self) -> impl Iterator<Item = (PermanentId, &Law)> {
        self.laws
            .iter()
            .filter(|l| l.status == LawStatus::InForce && l.kind == PolicyKind::KeepWatch)
            .filter_map(|l| Some((l.holder?, l)))
    }

    /// The one who lights the streets by a law in force, and that law (M10b slice DY, fourth
    /// part).
    pub fn lamplighter(&self) -> Option<(PermanentId, &Law)> {
        self.office(PolicyKind::LightStreets)
    }

    /// The one who keeps the school by a law in force, and that law (M10c slice EB, third
    /// part).
    pub fn schoolteacher(&self) -> Option<(PermanentId, &Law)> {
        self.office(PolicyKind::KeepSchool)
    }

    /// The one who strikes coin by a law in force, and that law (M7d slice CE).
    pub fn minter(&self) -> Option<(PermanentId, &Law)> {
        self.office(PolicyKind::Mint)
    }

    /// The holder of the office a law of kind `kind` in force names, and that law.
    fn office(&self, kind: PolicyKind) -> Option<(PermanentId, &Law)> {
        self.laws
            .iter()
            .filter(|l| l.status == LawStatus::InForce && l.kind == kind)
            .find_map(|l| Some((l.holder?, l)))
    }

    /// The laws in force of kind `kind` (by the catalog's policies).
    pub fn in_force<'a>(
        &'a self,
        policies: &'a [PolicyDef],
        kind: PolicyKind,
    ) -> impl Iterator<Item = &'a Law> + 'a {
        self.laws.iter().filter(move |l| {
            l.status == LawStatus::InForce
                && policies
                    .get(usize::from(l.policy))
                    .is_some_and(|d| d.kind == kind)
        })
    }

    /// Law `law` in words, as [`law_words`] says it, but a repeal naming the law of this polity
    /// it ends (M4c slice AI, step two): "an end to Ada's keeping of the common store".
    pub fn words_of(
        &self,
        law: &Law,
        policies: &[PolicyDef],
        name_of: &dyn Fn(PermanentId) -> String,
    ) -> String {
        let kind = policies.get(usize::from(law.policy)).map(|d| d.kind);
        let repeal = kind == Some(PolicyKind::Repeal);
        // A claim says how many places it names (M5c slice AT).
        if kind == Some(PolicyKind::ClaimPlace) {
            let n = self.claimed.iter().filter(|c| c.0 == law.id).count();
            return format!(
                "a claim on {} where people of another settlement worked: they may use {} \
                 only by leave",
                if n == 1 {
                    "a place".to_owned()
                } else {
                    format!("{n} places")
                },
                if n == 1 { "it" } else { "them" }
            );
        }
        // A crossing says how long a stream it spans and the work it asks (M5c slice AW).
        if kind == Some(PolicyKind::BuildCrossing)
            && let Some(site) = self.site_of(law.id)
        {
            return format!(
                "a crossing over the {:.1} m stream where its people wade, each household doing \
                 its share of {:.0} hours of work",
                site.span_m, site.labour_h
            );
        }
        // A well says how deep it would be dug and the work it asks (M6a slice BA).
        if kind == Some(PolicyKind::DigWell)
            && let Some(site) = self.well_site_of(law.id)
        {
            return format!(
                "a well {:.1} m deep beside the hearth, each household doing its share of {:.0} \
                 hours of work",
                site.target_m, site.labour_h
            );
        }
        // A bank says how long and high it would stand and the work it asks (M6b slice BG).
        if kind == Some(PolicyKind::BuildBank)
            && let Some(site) = self.bank_site_of(law.id)
        {
            return format!(
                "a bank {:.0} m long and {:.1} m high against floodwater, each household doing \
                 its share of {:.0} hours of work",
                site.length_m(),
                site.height_m,
                site.labour_h
            );
        }
        // A plan says its version, its grid and what it lays (M8c slice CT, second part).
        if kind == Some(PolicyKind::Plan)
            && let Some(plan) = self.plan_of(law.id)
        {
            let l = &plan.layout;
            // How its parcels are given, said only when not by need (M8c slice CU, second part).
            // An offer says what it asks: a fee, a yearly ground rent, or both (third part).
            let given = match policies
                .get(usize::from(law.policy))
                .and_then(|d| d.plan.as_ref())
            {
                Some(d) if d.allot == civ_land::AllotRule::Offer => {
                    let asks = match (d.fee.is_some(), d.rent.is_some()) {
                        (true, true) => " for a fee and a yearly ground rent",
                        (false, true) => " for a yearly ground rent",
                        _ => " for a fee",
                    };
                    format!(" given{asks}")
                }
                Some(d) if d.allot != civ_land::AllotRule::Need => {
                    format!(" given {}", d.allot.words())
                }
                _ => String::new(),
            };
            // A plan opening in phases says how many (fourth part), and one that takes or pays
            // for the fields under it, which (fifth part).
            let def = policies
                .get(usize::from(law.policy))
                .and_then(|d| d.plan.as_ref());
            let given = match def.filter(|d| d.phases > 1) {
                Some(d) => format!("{given}, opening in {} phases", d.phases),
                None => given,
            };
            let given = match def.map(|d| d.fields) {
                Some(FieldClause::Taken) => format!("{given}, taking the fields under it"),
                Some(FieldClause::Paid) => format!("{given}, paying for the fields under it"),
                _ => given,
            };
            return format!(
                "a plan (version {}) of {} by {} blocks of {:.0} by {:.0} m with {:.0} m streets \
                 running {}, laying out {} parcels{}{}, its streets{} kept open",
                plan.version,
                plan.grid.blocks.0,
                plan.grid.blocks.1,
                plan.grid.block_m.0,
                plan.grid.block_m.1,
                plan.grid.street_m,
                l.axis.map_or("east and west", civ_land::plans::Axis::name),
                l.parcels.len(),
                if l.sites.is_empty() {
                    ""
                } else {
                    " round a square"
                },
                given,
                if l.sites.is_empty() {
                    ""
                } else {
                    " and square"
                },
            );
        }
        // A passing place says the work it asks (M8b slice CQ, third part).
        if kind == Some(PolicyKind::BuildWay)
            && let Some(site) = self.way_site_of(law.id)
        {
            return format!(
                "a passing place cleared where carts wait, each household doing its share of {:.0} \
                 hours of work",
                site.labour_h
            );
        }
        // A line laid together says the work it asks (M10b slice DX, third part).
        if kind == Some(PolicyKind::BuildLine)
            && let Some(site) = self.way_site_of(law.id)
        {
            return format!(
                "a line of track laid together from a pit to the hearth, each household doing its \
                 share of {:.0} hours of work",
                site.labour_h
            );
        }
        // A building raised together says the work it asks (M8d slice DC).
        if kind == Some(PolicyKind::RaiseBuilding)
            && let Some(site) = self.raise_site_of(law.id)
            && let Some(d) = policies.get(usize::from(law.policy))
        {
            // Where it stands: finished, or stalled, kept visible (ADR-0031 §6).
            let state = match (site.finished, site.stalled) {
                (Some(_), _) => "; finished",
                (None, Some(_)) => "; stalled",
                (None, None) => "",
            };
            return format!(
                "{} raised together near the hearth, each household doing its share of {:.0} \
                 hours of work{state}",
                d.name.to_lowercase(),
                site.labour_h
            );
        }
        // A palisade says how long a line it would raise, the homes it takes in and the work it
        // asks (M6c slice BK).
        if kind == Some(PolicyKind::Enclose)
            && let Some(site) = self.enclosure_site_of(law.id)
        {
            return format!(
                "a palisade {:.0} m round the hearth with {} gate{}, taking in {} home{}, each \
                 household doing its share of {:.0} hours of work",
                site.length_m(),
                site.gates,
                if site.gates == 1 { "" } else { "s" },
                site.inside,
                if site.inside == 1 { "" } else { "s" },
                site.labour_h
            );
        }
        // Keeping a bank names it (M6b slice BG).
        if kind == Some(PolicyKind::KeepBank)
            && let Some((_, site)) = self.kept_bank_of(law.id)
        {
            return format!(
                "the keeping of the bank {:.0} m long against floodwater, looked over after each \
                 high water and each year and mended whenever it wants it, each household doing \
                 its share of the work",
                site.length_m()
            );
        }
        match law
            .ends
            .filter(|_| repeal)
            .and_then(|e| self.laws.iter().find(|l| l.id == e))
        {
            Some(ended) => format!("an end to {}", ending_words(ended, policies, name_of)),
            None => law_words(law, policies, name_of),
        }
    }

    /// The law with id `id`.
    pub fn law_mut(&mut self, id: PermanentId) -> Option<&mut Law> {
        self.laws.iter_mut().find(|l| l.id == id)
    }

    /// Spoils the store up to `t` as anyone's store spoils, under a roof when `sheltered`.
    pub fn settle_stores(&mut self, t: SimTime, goods: &[crate::params::GoodDef], sheltered: bool) {
        if t <= self.stores_at {
            return;
        }
        self.stores.resize(goods.len(), 0.0);
        let days = (t.minutes() - self.stores_at.minutes()) as f64 / 1440.0;
        let mut spoiled_kcal = 0.0;
        for (g, (kg, d)) in self.stores.iter_mut().zip(goods).enumerate() {
            let half_life = if sheltered && d.sheltered_half_life_days > 0.0 {
                d.sheltered_half_life_days
            } else {
                d.half_life_days
            };
            if half_life > 0.0 && *kg > 0.0 {
                let after = *kg * 0.5f64.powf(days / half_life);
                self.flows.add(crate::person::Flow::Spoiled, g, *kg - after);
                spoiled_kcal += (*kg - after) * d.kcal_per_kg;
                *kg = after;
            }
        }
        self.stores_at = t;
        // Measured against what a storehouse would answer (M8d slice DC).
        if spoiled_kcal > 0.0 {
            self.measured_mut(t.date().year).spoiled_kcal += spoiled_kcal;
        }
    }
}

/// What a household expects of its coming year, in kcal: what it holds now, what its next
/// harvest should bring in an ordinary year, and a year's need.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outlook {
    /// Food held now, kcal.
    pub held: f64,
    /// Its next harvest in an ordinary year, kcal.
    pub harvest: f64,
    /// A year of its members' food, kcal.
    pub year_need: f64,
}

/// What a settlement's people believe about their years (ADR-0013 §5: forecasts from what they
/// have seen).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Belief {
    /// The chance a year is lean: the lean years the settlement has seen, with the prior.
    pub p_lean: f64,
    /// An ordinary harvest of one of its households, on average, kcal.
    pub mean_harvest: f64,
}

/// Below this a year's food (as a share of the need, above subsistence) counts as nearly
/// nothing: it keeps the log finite and the order of worse outcomes.
const FLOOR_SMOOTH: f64 = 0.05;

/// The chance a year is lean, believed by people who have seen `lean` lean years in `years`.
pub fn p_lean(lean: f64, years: f64, params: &PolityParams) -> f64 {
    ((lean + params.prior_lean) / (years + params.prior_years).max(1e-9)).clamp(0.0, 1.0)
}

/// How much better (or worse) a household expects its year under a common store with levy share
/// `levy` than without one: the change in the expected log of its year's food above subsistence,
/// over an ordinary year and a lean one (one whose harvest is `lean_harvest` of an ordinary
/// one). It pays the levy on its harvest; the store holds the levy on an ordinary harvest of the
/// average household, and gives back what brings it up to a year's need, as far as that goes.
/// Food near subsistence is worth the most, so a household that would go hungry in a lean year
/// gains, and one whose harvest is large loses (research 09-17 §1.2: households insure one
/// another, though a year lean for all can overwhelm it).
pub fn store_gain(o: &Outlook, b: &Belief, levy: f64, params: &PolityParams) -> f64 {
    let need = o.year_need.max(1.0);
    let u = |kcal: f64| ((kcal / need - params.subsistence_share).max(0.0) + FLOOR_SMOOTH).ln();
    let pool = (levy * b.mean_harvest).max(0.0);
    let year = |harvest: f64| {
        let before = o.held + harvest;
        let after = before - levy * harvest;
        let relief = (need - after).clamp(0.0, pool);
        u(after + relief) - u(before)
    };
    let p = b.p_lean.clamp(0.0, 1.0);
    (1.0 - p) * year(o.harvest) + p * year(o.harvest * params.lean_harvest)
}

/// How much better a household expects its year when `kcal` more food is kept for it, in the same
/// units as [`store_gain`]: what a store kept under a roof saves from spoiling, as its share.
pub fn kept_gain(o: &Outlook, kcal: f64, params: &PolityParams) -> f64 {
    let need = o.year_need.max(1.0);
    let u = |kcal: f64| ((kcal / need - params.subsistence_share).max(0.0) + FLOOR_SMOOTH).ln();
    let before = o.held + o.harvest;
    u(before + kcal.max(0.0)) - u(before)
}

/// What a household expects of a law against taking (M4b slice AB), in the units of
/// [`store_gain`]: `recover` kcal a year back to it (what it lost to takers it knows of in the past
/// year, with the bundle's compensation), less `owe` kcal a year from it (what its own members
/// took, with the bundle, at the chance they believe a taker runs of being seen).
pub fn against_gain(o: &Outlook, recover: f64, owe: f64, params: &PolityParams) -> f64 {
    kept_gain(o, recover, params) - levy_cost(o, owe, params)
}

/// What paying a levy of `kcal` now costs a household with outlook `o`, in the same units as
/// [`store_gain`]: the fall in the log of its year's food above subsistence.
pub fn levy_cost(o: &Outlook, kcal: f64, params: &PolityParams) -> f64 {
    let need = o.year_need.max(1.0);
    let u = |kcal: f64| ((kcal / need - params.subsistence_share).max(0.0) + FLOOR_SMOOTH).ln();
    let before = o.held + o.harvest;
    u(before) - u(before - kcal.max(0.0))
}

/// Whether paying a levy of `kcal` now would leave a household holding `held` kcal below
/// subsistence for a year of `year_need`: then it cannot pay (research 09-01 §6.4: being unable
/// is not refusing).
pub fn cannot_pay(held: f64, kcal: f64, year_need: f64, params: &PolityParams) -> bool {
    held - kcal < params.subsistence_share * year_need
}

/// A member's stance: their household's forecast in points, and their regard for the sponsor
/// (0-1), against the margin (research 09-05 §1.4; ADR-0013 §3, stage 2).
pub fn stance(gain_points: f64, regard: f64, params: &PolityParams) -> (Stance, f64) {
    stance_between(gain_points, regard, 0.0, params)
}

/// As [`stance`], for a law that would put someone in the place of an office's holder (M4c slice
/// AH: a petition against an office): regard for the one it names, 0 to 1, less what holds them
/// to the one it would replace, -1 to 1 (regard for them, less a grievance against them). Its
/// regard points may be below nothing.
pub fn stance_between(
    gain_points: f64,
    regard: f64,
    replaced: f64,
    params: &PolityParams,
) -> (Stance, f64) {
    let regard_points = params.w_regard * (regard.clamp(0.0, 1.0) - replaced.clamp(-1.0, 1.0));
    let s = gain_points + regard_points;
    let stance = if s > params.stance_margin {
        Stance::Support
    } else if s < -params.stance_margin {
        Stance::Oppose
    } else {
        Stance::Abstain
    };
    (stance, regard_points)
}

/// The points for paying a levy: `norm`, what the norms the person holds add for abiding by what
/// the gathering decided (M4c slice AG), the stance the person took, their regard for the law's
/// sponsor, less what paying costs their household. There is no sanction: nobody is set to see
/// who pays.
pub fn comply_points(
    norm: f64,
    stance: f64,
    regard: f64,
    cost_points: f64,
    params: &PolityParams,
) -> f64 {
    params.comply_base + norm + params.w_stance * stance + params.w_regard * regard.clamp(0.0, 1.0)
        - cost_points
}

/// The chance someone pays given [`comply_points`]: a logistic choice between paying and
/// keeping it back.
pub fn comply_chance(points: f64) -> f64 {
    1.0 / (1.0 + (-points).exp())
}

/// A move a person could make (ADR-0013 §5): proposing a policy at one level against an issue,
/// with what they forecast of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveOption {
    /// The template, by index in the catalog's policies.
    pub policy: u16,
    /// The levy share.
    pub levy_share: f64,
    /// The issue it answers.
    pub issue: IssueKind,
    /// The one it would name, for a law that names someone.
    pub nominee: Option<PermanentId>,
    /// The bundle, for a law against taking.
    pub sanction: Sanction,
    /// The hours, for a curfew.
    pub hours: Hours,
    /// The body, for an amendment of the custom (M4c slice AF).
    pub body: Option<Body>,
    /// The law in force it would replace, for one a faction petitions for (M4c slice AH).
    pub ends: Option<PermanentId>,
    /// The river cell it would span, for a crossing (M5c slice AW).
    pub site: Option<u32>,
    /// The settlement it goes against, for a band sent by law (M6c slice BJ).
    pub target: Option<PermanentId>,
    /// The forecast for the person's own household ([`store_gain`]).
    pub own_gain: f64,
    /// The forecast for the households of those who regard them, weighted by that regard.
    pub followers_gain: f64,
    /// The share of those who would take a stance whom they expect to back it, from those they
    /// know.
    pub support: f64,
}

/// A deliberator's choice and its receipt.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    /// The move made, by index in the options; `None` for making none.
    pub chosen: Option<usize>,
    /// Each option's score, points, in order; then the outside option's.
    pub scores: Vec<f64>,
}

/// How a person who may make institutional moves chooses among them (ADR-0013 §5): from a copy
/// of the facts and the moves available, a choice and its receipt. A rule-based one now; another
/// (an LLM's, plan §8) can stand in its place.
pub trait Deliberator {
    /// Chooses among `moves` or none, with `u` a keyed draw in 0-1.
    fn choose(&self, moves: &[MoveOption], u: f64) -> Choice;
}

/// The rule-based deliberator (ADR-0013 §5): one level deep. A move is worth what the person
/// forecasts for their household and those who regard them, by the chance it passes, less what
/// proposing costs; making none is worth nothing. A softmax picks one.
#[derive(Clone, Copy, Debug)]
pub struct RuleDeliberator<'a> {
    /// The tuning.
    pub params: &'a PolityParams,
}

impl RuleDeliberator<'_> {
    /// A move's score, points.
    pub fn score(&self, m: &MoveOption) -> f64 {
        let p = self.params;
        let gain = m.own_gain + p.w_followers * m.followers_gain;
        m.support.clamp(0.0, 1.0) * p.w_gain * gain - p.propose_cost
    }
}

impl Deliberator for RuleDeliberator<'_> {
    fn choose(&self, moves: &[MoveOption], u: f64) -> Choice {
        let mut scores: Vec<f64> = moves.iter().map(|m| self.score(m)).collect();
        scores.push(0.0);
        let t = self.params.temperature.max(1e-6);
        let scaled: Vec<f64> = scores.iter().map(|s| s / t).collect();
        let picked = crate::demography::pick_softmax(&scaled, u);
        Choice {
            chosen: picked.filter(|&i| i < moves.len()),
            scores,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn params() -> PolityParams {
        PolityParams::core()
    }

    fn pid(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn what_was_decided_is_told_under_the_body_that_decided_it() {
        let t = |day: i64| SimTime::from_minutes(day * 24 * 60);
        let mut polity = Polity::found(pid(1), pid(2), t(0), &params());
        let elders = Body {
            members: Membership::Elders,
            ..polity.body
        };
        // An amendment decided on day 100 makes the elders the body from then.
        polity.versions.push(CustomVersion {
            body: elders,
            since: t(100),
            law: Some(pid(9)),
            seized_by: None,
        });
        polity.body = elders;
        assert_eq!(polity.body_at(t(50)).members, Membership::Adults);
        assert_eq!(
            polity.body_at(t(100)).members,
            Membership::Adults,
            "the amendment itself was decided by the adults"
        );
        assert_eq!(polity.body_at(t(101)).members, Membership::Elders);
    }

    #[test]
    fn the_founding_custom_decides_by_acclamation_and_records_its_failures() {
        let body = Body::gathering(&params());
        assert_eq!(body.quorum(40), 10);
        assert_eq!(body.quorum(1), 1);
        assert_eq!(body.decide(40, 12, 7, 5), Outcome::Passed);
        assert_eq!(body.decide(40, 12, 5, 7), Outcome::Failed);
        assert_eq!(body.decide(40, 12, 6, 6), Outcome::Tied, "a tie fails");
        assert_eq!(
            body.decide(40, 9, 9, 0),
            Outcome::NoQuorum,
            "too few came, however they stood"
        );
    }

    #[test]
    fn pooling_helps_the_household_that_would_go_hungry_and_costs_the_one_with_plenty() {
        let p = params();
        let need = 365.0 * 4.0 * 2100.0;
        let belief = Belief {
            p_lean: 0.25,
            mean_harvest: 1.2 * need,
        };
        let poor = Outlook {
            held: 0.05 * need,
            harvest: 0.8 * need,
            year_need: need,
        };
        let rich = Outlook {
            held: 0.1 * need,
            harvest: 2.0 * need,
            year_need: need,
        };
        let gain = |o: &Outlook| store_gain(o, &belief, 0.1, &p);
        assert!(gain(&poor) > 0.1, "{}", gain(&poor));
        assert!(gain(&rich) < -0.05, "{}", gain(&rich));
        // No levy, no change.
        assert!(store_gain(&poor, &belief, 0.0, &p).abs() < 1e-12);
        // The more lean years seen, the more a middling household gains.
        let middling = Outlook {
            held: 0.1 * need,
            harvest: 1.2 * need,
            year_need: need,
        };
        let at = |p_lean| store_gain(&middling, &Belief { p_lean, ..belief }, 0.1, &p);
        assert!(at(0.5) > at(0.25) && at(0.25) > at(0.1));
    }

    #[test]
    fn lean_years_seen_raise_the_belief_from_its_prior() {
        let p = params();
        assert!((p_lean(0.0, 0.0, &p) - 0.25).abs() < 1e-12);
        assert!(p_lean(2.0, 4.0, &p) > 0.25);
        assert!(p_lean(0.0, 20.0, &p) < 0.1);
    }

    #[test]
    fn a_stance_weighs_the_forecast_and_regard_for_the_sponsor() {
        let p = params();
        assert_eq!(stance(1.0, 0.0, &p).0, Stance::Support);
        assert_eq!(stance(-1.0, 0.0, &p).0, Stance::Oppose);
        assert_eq!(stance(0.1, 0.0, &p).0, Stance::Abstain);
        // Regard for the sponsor carries one who would lose a little.
        let (s, r) = stance(-0.5, 1.0, &p);
        assert_eq!((s, r), (Stance::Support, 1.0));
        assert_eq!(stance(-0.5, 5.0, &p).1, 1.0, "regard counts up to 1");
        // Someone named in place of an office's holder (M4c slice AH): regard for the holder
        // weighs against it, and a grievance against them lowers that regard below nothing.
        assert_eq!(
            stance_between(0.0, 0.5, 0.5, &p),
            (Stance::Abstain, 0.0),
            "liked alike"
        );
        assert_eq!(stance_between(0.0, 0.0, 1.0, &p).0, Stance::Oppose);
        let (s, r) = stance_between(0.0, 0.5, 0.5 - 1.0, &p);
        assert_eq!((s, r), (Stance::Support, p.w_regard));
    }

    #[test]
    fn paying_is_likelier_for_those_who_backed_the_law_and_dearer_for_the_hard_pressed() {
        let p = params();
        let backed = comply_chance(comply_points(1.5, 1.0, 0.0, 0.5, &p));
        let opposed = comply_chance(comply_points(1.5, -1.0, 0.0, 0.5, &p));
        let pressed = comply_chance(comply_points(1.5, 1.0, 0.0, 3.0, &p));
        let unbound = comply_chance(comply_points(0.0, 1.0, 0.0, 0.5, &p));
        assert!(backed > opposed && backed > pressed && backed > unbound);
        assert!(cannot_pay(100.0, 60.0, 200.0, &p));
        assert!(!cannot_pay(200.0, 60.0, 200.0, &p));
    }

    #[test]
    fn a_law_keeps_who_knows_it_in_order() {
        let mut law = Law {
            id: pid(1),
            policy: 0,
            kind: PolicyKind::CommonStore,
            levy_share: 0.1,
            holder: None,
            relief_days: 5.0,
            sanction: Default::default(),
            hours: (0, 0),
            status: LawStatus::Proposed,
            sponsor: pid(7),
            proposed: SimTime::ZERO,
            issue: IssueKind::FoodShort,
            meets_day: 1,
            decided: None,
            outcome: None,
            eligible: 0,
            stances: Vec::new(),
            known: Vec::new(),
            compliance: Compliance::default(),
            watch: WatchRecord::default(),
            body: None,
            ends: None,
            agreement: None,
            target: None,
        };
        assert!(law.learn(pid(9), 3));
        assert!(law.learn(pid(4), 5));
        assert!(!law.learn(pid(9), 6), "known already");
        assert_eq!(law.known, vec![(pid(4), 5), (pid(9), 3)]);
        assert!(law.knows(pid(4)) && !law.knows(pid(7)));
    }

    #[test]
    fn a_law_in_words_follows_the_gatherings_verbs() {
        let def = |kind: PolicyKind, name: &str| PolicyDef {
            id: format!("core:policy/{name}"),
            name: name.to_owned(),
            description: String::new(),
            kind,
            answers: Vec::new(),
            levy_shares: Vec::new(),
            relief_days: 0.0,
            bundles: Vec::new(),
            hours: Vec::new(),
            bodies: Vec::new(),
            question: None,
            bears: Vec::new(),
            rule: None,
            raise: None,
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
            licence: None,
            keep_apart: None,
            holder_needs: None,
            register: None,
            middens: None,
        };
        let policies = [
            def(PolicyKind::CommonStore, "Common store"),
            def(PolicyKind::KeepStore, "Storekeeper"),
            def(PolicyKind::AgainstTaking, "Against taking"),
            def(PolicyKind::KeepWatch, "Watch"),
        ];
        let mut law = Law {
            id: pid(1),
            policy: 0,
            kind: PolicyKind::CommonStore,
            levy_share: 0.1,
            holder: None,
            relief_days: 5.0,
            sanction: Default::default(),
            hours: (0, 0),
            status: LawStatus::Proposed,
            sponsor: pid(7),
            proposed: SimTime::ZERO,
            issue: IssueKind::FoodShort,
            meets_day: 1,
            decided: None,
            outcome: None,
            eligible: 0,
            stances: Vec::new(),
            known: Vec::new(),
            compliance: Compliance::default(),
            watch: WatchRecord::default(),
            body: None,
            ends: None,
            agreement: None,
            target: None,
        };
        let name_of = |_: PermanentId| "Ada".to_owned();
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "a common store, taking a tenth of each harvest"
        );
        // A petition's demand replaces the store in force (M4c slice AH).
        law.ends = Some(pid(2));
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "a common store, taking a tenth of each harvest in place of the levy in force"
        );
        law.levy_share = 0.0;
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "an end to the common store's levy (the store gives what it holds)"
        );
        law.ends = None;
        law.policy = 1;
        law.holder = Some(pid(9));
        // "proposed …", "agreed to …" and "turned down …" all read.
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "Ada as keeper of the common store"
        );
        law.policy = 2;
        law.holder = None;
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "a law against taking: whoever is found to have taken from another household's store \
             gives back what was taken"
        );
        law.sanction = Sanction {
            compensation_days: 3.0,
            fine_days: 1.0,
            ..Default::default()
        };
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "a law against taking: whoever is found to have taken from another household's store \
             gives back what was taken, with three days' food to the household taken from and one \
             day's to the common store"
        );
        law.sanction = Sanction {
            exile: true,
            ..Default::default()
        };
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "a law against taking: whoever is found to have taken from another household's store \
             gives back what was taken, and is sent from the valley"
        );
        law.policy = 3;
        law.holder = Some(pid(9));
        assert_eq!(
            law_words(&law, &policies, &name_of),
            "Ada to keep watch over the stores at night"
        );
    }

    #[test]
    fn a_watch_office_reads_with_what_anyone_could_see_of_it() {
        let policies = [PolicyDef {
            id: "core:policy/keep_watch".to_owned(),
            name: "Watch".to_owned(),
            description: String::new(),
            kind: PolicyKind::KeepWatch,
            answers: vec![IssueKind::Takings],
            levy_shares: Vec::new(),
            relief_days: 0.0,
            bundles: Vec::new(),
            hours: Vec::new(),
            bodies: Vec::new(),
            question: None,
            bears: Vec::new(),
            rule: None,
            raise: None,
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
            licence: None,
            keep_apart: None,
            holder_needs: None,
            register: None,
            middens: None,
        }];
        let mut polity = Polity::found(pid(1), pid(2), SimTime::ZERO, &params());
        polity.laws.push(Law {
            id: pid(3),
            policy: 0,
            kind: PolicyKind::KeepWatch,
            levy_share: 0.0,
            holder: Some(pid(9)),
            relief_days: 0.0,
            sanction: Sanction::default(),
            hours: (0, 0),
            status: LawStatus::InForce,
            sponsor: pid(9),
            proposed: SimTime::ZERO,
            issue: IssueKind::Takings,
            meets_day: 0,
            decided: Some(SimTime::ZERO),
            outcome: Some(Outcome::Passed),
            eligible: 0,
            stances: Vec::new(),
            known: Vec::new(),
            compliance: Compliance::default(),
            watch: WatchRecord {
                rounds: 14,
                minutes: 1800.0,
                cases: 1,
                ..Default::default()
            },
            body: None,
            ends: None,
            agreement: None,
            target: None,
        });
        assert_eq!(polity.watcher().map(|w| w.0), Some(pid(9)));
        assert!(polity.keeper().is_none(), "a watch keeps no store");
        let words = office_words(
            &polity,
            &policies,
            &|_| "Bram".to_owned(),
            &|_| String::new(),
            0,
        );
        assert_eq!(words.len(), 1);
        assert!(
            words[0].starts_with("Watch: Bram, since ")
                && words[0].ends_with("; 14 rounds, 30 hours on them, 1 case brought"),
            "{}",
            words[0]
        );
    }

    #[test]
    fn a_law_against_taking_helps_those_taken_from_and_costs_those_who_took() {
        let p = params();
        let need_day = 4.0 * 2100.0;
        let o = Outlook {
            held: 30.0 * need_day,
            harvest: 300.0 * need_day,
            year_need: 365.0 * need_day,
        };
        let bundle = Sanction {
            compensation_days: 3.0,
            fine_days: 3.0,
            ..Default::default()
        };
        let robbed = crate::crime::TakingsKnown {
            lost_kcal: 10.0 * need_day,
            lost: 2,
            risk: 0.5,
            ..Default::default()
        };
        let took = crate::crime::TakingsKnown {
            took_kcal: 10.0 * need_day,
            took: 2,
            risk: 0.5,
            ..Default::default()
        };
        let gain = |k: &crate::crime::TakingsKnown, s: &Sanction| {
            let (recover, owe) = k.under(s, need_day, 60.0);
            against_gain(&o, recover, owe, &p)
        };
        assert!(gain(&robbed, &bundle) > 0.0);
        assert!(gain(&took, &bundle) < 0.0);
        // A harsher bundle is worth more to the one taken from and costs the taker more.
        assert!(gain(&robbed, &bundle) > gain(&robbed, &Sanction::default()));
        assert!(gain(&took, &bundle) < gain(&took, &Sanction::default()));
        // Exile weighs on a taker's household as the days of food a grown member is worth to it,
        // and brings the one taken from nothing more.
        let exiled = Sanction {
            exile: true,
            ..bundle
        };
        assert!(gain(&took, &exiled) < gain(&took, &bundle));
        assert!((gain(&robbed, &exiled) - gain(&robbed, &bundle)).abs() < 1e-12);
        // A taker sure of never being seen expects to owe nothing.
        let unseen = crate::crime::TakingsKnown { risk: 0.0, ..took };
        assert!(gain(&unseen, &bundle).abs() < 1e-12);
        // A household with no takings either way is untouched.
        let none = crate::crime::TakingsKnown {
            risk: 0.5,
            ..Default::default()
        };
        assert!(gain(&none, &bundle).abs() < 1e-12);
        // What takers of another settlement took no finding at home recovers, and what one's
        // own took elsewhere no gathering at home hears (M6c slice BI).
        let raided = crate::crime::TakingsKnown {
            outside_kcal: robbed.lost_kcal,
            outside: robbed.lost,
            ..robbed
        };
        assert!(gain(&raided, &bundle).abs() < 1e-12);
        let raider = crate::crime::TakingsKnown {
            took_outside_kcal: took.took_kcal,
            took_outside: took.took,
            ..took
        };
        assert!(gain(&raider, &bundle).abs() < 1e-12);
    }

    #[test]
    fn a_curfews_hours_run_past_midnight_when_they_end_before_they_begin() {
        let h = |hour: i64| hour * 60;
        assert!(within_hours((21, 5), h(21)));
        assert!(within_hours((21, 5), h(23) + 59));
        assert!(within_hours((21, 5), h(0)));
        assert!(within_hours((21, 5), h(4) + 59));
        assert!(!within_hours((21, 5), h(5)));
        assert!(!within_hours((21, 5), h(20) + 59));
        assert!(within_hours((9, 17), h(12)) && !within_hours((9, 17), h(17)));
        // No hours, no curfew.
        assert!(!within_hours((0, 0), h(3)));
        assert_eq!(hours_long((21, 5)), 8);
        assert_eq!(hours_long((23, 4)), 5);
        assert_eq!(hours_long((9, 17)), 8);
        assert_eq!(hours_long((0, 0)), 0);
    }

    #[test]
    fn a_two_thirds_rule_needs_two_of_every_three_who_take_a_side() {
        let body = Body {
            pass: PassRule::TwoThirds,
            ..Body::gathering(&params())
        };
        assert_eq!(body.decide(40, 12, 8, 4), Outcome::Passed);
        assert_eq!(
            body.decide(40, 12, 7, 5),
            Outcome::Failed,
            "a bare majority is not enough"
        );
        assert_eq!(body.decide(40, 12, 6, 6), Outcome::Tied);
        assert_eq!(body.decide(40, 9, 9, 0), Outcome::NoQuorum);
        for m in Membership::ALL {
            assert_eq!(Membership::from_code(m.code()), Some(m));
            assert_eq!(Membership::from_name(m.name()), Some(m));
        }
        for r in PassRule::ALL {
            assert_eq!(PassRule::from_code(r.code()), Some(r));
            assert_eq!(PassRule::from_name(r.name()), Some(r));
        }
    }

    #[test]
    fn the_custom_says_who_decides_and_keeps_its_versions() {
        let p = params();
        let mut polity = Polity::found(pid(1), pid(2), SimTime::from_minutes(0), &p);
        assert_eq!(polity.versions.len(), 1);
        let elders = Body {
            members: Membership::Elders,
            quorum_share: 0.5,
            ..polity.body
        };
        assert_eq!(
            elders.clause(),
            "the elders of its households who come to the hearth decide by acclamation: more \
             for than against carries it, and a tie fails; half of them must come"
        );
        let law = Law {
            id: pid(5),
            policy: 0,
            kind: PolicyKind::AmendBody,
            levy_share: 0.0,
            holder: None,
            relief_days: 0.0,
            sanction: Sanction::default(),
            hours: (0, 0),
            status: LawStatus::InForce,
            sponsor: pid(9),
            proposed: SimTime::from_minutes(0),
            issue: IssueKind::Overruled,
            meets_day: 1,
            decided: Some(SimTime::from_minutes(1440)),
            outcome: Some(Outcome::Passed),
            eligible: 0,
            stances: Vec::new(),
            known: Vec::new(),
            compliance: Compliance::default(),
            watch: WatchRecord::default(),
            body: Some(elders),
            ends: None,
            agreement: None,
            target: None,
        };
        let policies = [PolicyDef {
            id: "core:policy/amend_custom".to_owned(),
            name: "Amend the custom".to_owned(),
            description: String::new(),
            kind: PolicyKind::AmendBody,
            answers: vec![IssueKind::Overruled],
            levy_shares: Vec::new(),
            relief_days: 0.0,
            bundles: Vec::new(),
            hours: Vec::new(),
            bodies: vec![elders],
            question: None,
            bears: Vec::new(),
            rule: None,
            raise: None,
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
            licence: None,
            keep_apart: None,
            holder_needs: None,
            register: None,
            middens: None,
        }];
        assert!(
            law_words(&law, &policies, &|_| "Ada".to_owned())
                .starts_with("a change of the custom: the elders of its households")
        );
        polity.laws.push(law);
        polity.body = elders;
        polity.versions.push(CustomVersion {
            body: elders,
            since: SimTime::from_minutes(1440),
            law: Some(pid(5)),
            seized_by: None,
        });
        let history = custom_history_words(&polity, &|_| "Ada".to_owned());
        assert_eq!(history.len(), 2);
        assert!(
            history[0].contains("the founding custom: the adults"),
            "{history:?}"
        );
        assert!(
            history[1].contains("by the amendment Ada proposed: the elders"),
            "{history:?}"
        );
    }

    #[test]
    fn the_rule_deliberator_proposes_what_it_expects_to_pass_and_gain_from() {
        let p = params();
        let d = RuleDeliberator { params: &p };
        let good = MoveOption {
            policy: 0,
            levy_share: 0.1,
            issue: IssueKind::FoodShort,
            nominee: None,
            sanction: Sanction::default(),
            hours: (0, 0),
            body: None,
            ends: None,
            site: None,
            target: None,
            own_gain: 0.3,
            followers_gain: 0.2,
            support: 0.8,
        };
        let hopeless = MoveOption {
            support: 0.0,
            ..good
        };
        // Worth 0.8 × 10 × 0.4 − 0.5 = 2.7 points against nothing's 0: nearly always chosen.
        let c = d.choose(&[hopeless, good], 0.5);
        assert_eq!(c.chosen, Some(1));
        assert!((c.scores[1] - 2.7).abs() < 1e-9 && c.scores[2] == 0.0);
        // A move nobody would back costs more than it could bring.
        let c = d.choose(&[hopeless], 0.5);
        assert_eq!(c.chosen, None);
    }
}
