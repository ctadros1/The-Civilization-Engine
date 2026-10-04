//! Property regimes (`kind = "regime"`): who holds, works, lets and inherits land (ADR-0007 §2).
//! A world is created under one; which ground anyone breaks, works or lets is decided by people
//! at run time.

use civ_agents::params::{LandHolder, LandUse, LeaseRules, RegimeDef, Succession};
use serde::Deserialize;

/// The `kind` value of a regime.
pub const KIND: &str = "regime";
/// The id segment: `pack:regime/name`.
pub const ID_KIND: &str = "regime";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegimeFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub description: String,
    /// Exactly one regime is the default.
    pub default: bool,
    pub land: Land,
    pub succession: SuccessionTable,
    pub lease: Lease,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Land {
    /// `breaker` or `settlement`.
    pub holder: String,
    /// `holder` or `need`.
    #[serde(rename = "use")]
    pub land_use: String,
    /// Under `need`, the yearly review: `[month, day]`.
    pub review: [u16; 2],
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SuccessionTable {
    /// `heir`, `divided` or `settlement`.
    pub holdings: String,
    /// A new couple's household takes a share of its families' fields.
    pub union_share: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Lease {
    pub allowed: bool,
    /// The holder's share of the grain threshed from a let field.
    pub holder_share: f64,
    /// Crop years a lease runs.
    pub term_years: u32,
}

/// Days before the first of each month in the engine's 365-day calendar.
const MONTH_START: [u16; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
const MONTH_DAYS: [u16; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

impl RegimeFile {
    /// The compiled regime (call after [`RegimeFile::problems`] found none).
    pub fn def(&self) -> RegimeDef {
        let [month, day] = self.land.review;
        let m = usize::from(month.clamp(1, 12)) - 1;
        RegimeDef {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            is_default: self.default,
            holder: LandHolder::from_name(&self.land.holder).unwrap_or(LandHolder::Breaker),
            land_use: LandUse::from_name(&self.land.land_use).unwrap_or(LandUse::Holder),
            review_day: MONTH_START[m] + day.clamp(1, MONTH_DAYS[m]) - 1,
            succession: Succession::from_name(&self.succession.holdings)
                .unwrap_or(Succession::Heir),
            union_share: self.succession.union_share,
            lease: self.lease.allowed.then_some(LeaseRules {
                holder_share: self.lease.holder_share,
                term_years: self.lease.term_years,
            }),
        }
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        let names = |all: &[&str]| all.join("`, `");
        if LandHolder::from_name(&self.land.holder).is_none() {
            let all: Vec<&str> = LandHolder::ALL.iter().map(|v| v.name()).collect();
            p.push(format!("`land.holder` must be one of `{}`", names(&all)));
        }
        if LandUse::from_name(&self.land.land_use).is_none() {
            let all: Vec<&str> = LandUse::ALL.iter().map(|v| v.name()).collect();
            p.push(format!("`land.use` must be one of `{}`", names(&all)));
        }
        if Succession::from_name(&self.succession.holdings).is_none() {
            let all: Vec<&str> = Succession::ALL.iter().map(|v| v.name()).collect();
            p.push(format!(
                "`succession.holdings` must be one of `{}`",
                names(&all)
            ));
        }
        let [month, day] = self.land.review;
        if !(1..=12).contains(&month) || day < 1 || day > MONTH_DAYS[usize::from(month) - 1] {
            p.push(format!(
                "`land.review` must be a [month, day] of the 365-day calendar (got [{month}, {day}])"
            ));
        }
        let (holder, land_use) = (
            LandHolder::from_name(&self.land.holder),
            LandUse::from_name(&self.land.land_use),
        );
        match (holder, land_use) {
            (Some(LandHolder::Settlement), Some(LandUse::Holder)) => p.push(
                "a settlement cannot work the ground it holds: `land.holder = \"settlement\"` \
                 needs `land.use = \"need\"`"
                    .to_owned(),
            ),
            (Some(LandHolder::Breaker), Some(LandUse::Need)) => p.push(
                "only a settlement gives out fields by need: `land.use = \"need\"` needs \
                 `land.holder = \"settlement\"`"
                    .to_owned(),
            ),
            _ => {}
        }
        if self.lease.allowed && land_use == Some(LandUse::Need) {
            p.push(
                "fields given out by need are not let: `lease.allowed` needs `land.use = \"holder\"`"
                    .to_owned(),
            );
        }
        let share = self.lease.holder_share;
        if !(share.is_finite() && (0.0..1.0).contains(&share)) {
            p.push(format!(
                "`lease.holder_share` must be at least 0 and below 1 (got {share})"
            ));
        }
        if self.lease.term_years == 0 {
            p.push("`lease.term_years` must be at least 1".to_owned());
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(holder: &str, land_use: &str, holdings: &str, review: [u16; 2]) -> RegimeFile {
        RegimeFile {
            kind: KIND.to_owned(),
            id: "core:regime/test".to_owned(),
            name: "Test".to_owned(),
            description: "A test.".to_owned(),
            default: true,
            land: Land {
                holder: holder.to_owned(),
                land_use: land_use.to_owned(),
                review,
            },
            succession: SuccessionTable {
                holdings: holdings.to_owned(),
                union_share: true,
            },
            lease: Lease {
                allowed: land_use == "holder",
                holder_share: 0.25,
                term_years: 1,
            },
        }
    }

    #[test]
    fn a_regime_compiles_its_rules() {
        let f = file("settlement", "need", "settlement", [2, 1]);
        assert!(f.problems().is_empty(), "{:?}", f.problems());
        let d = f.def();
        assert_eq!(
            (d.holder, d.land_use, d.succession),
            (
                LandHolder::Settlement,
                LandUse::Need,
                Succession::Settlement
            )
        );
        assert_eq!(d.review_day, 31, "1 February");
        assert_eq!(d.lease, None, "fields given out by need are not let");
        let d = file("breaker", "holder", "divided", [2, 15]).def();
        assert_eq!(
            d.lease,
            Some(LeaseRules {
                holder_share: 0.25,
                term_years: 1
            })
        );
    }

    #[test]
    fn rules_that_cannot_go_together_are_refused() {
        let p = file("settlement", "holder", "settlement", [2, 1]).problems();
        assert!(p.iter().any(|m| m.contains("cannot work")), "{p:?}");
        let p = file("breaker", "need", "divided", [2, 1]).problems();
        assert!(p.iter().any(|m| m.contains("only a settlement")), "{p:?}");
        let mut f = file("settlement", "need", "settlement", [2, 1]);
        f.lease.allowed = true;
        assert!(f.problems().iter().any(|m| m.contains("not let")));
    }

    #[test]
    fn unknown_rules_and_impossible_dates_are_refused() {
        let f = file("chief", "lottery", "eldest", [2, 30]);
        let p = f.problems();
        assert_eq!(p.len(), 4, "{p:?}");
        assert!(p[0].contains("`breaker`, `settlement`"), "{p:?}");
    }
}
