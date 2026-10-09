//! Values (M4c slice AG, step three, ADR-0016 §4; research 06-04 §1.1, §6.1): a few slow content
//! axes, such as safety from want and harm or a household's say over what is its own, that weigh
//! outcomes not reducible to a household's own food (06-04 §1.1). Not to be confused with
//! [`crate::value`], what goods are worth to a household.
//!
//! Each person holds each value somewhere between −1 (cares for it less than most) and 1 (more
//! than most), drawn by a key and pulled toward their parents' as the objection to taking is
//! (06-04 §1.7: transmission with variation). A policy template says how a law of its kind bears
//! on each value, and what a person makes of a law is their household's lot and what it does to
//! what they hold dear: their anchor on its question, their stance at its gathering and how they
//! weigh proposing it. Values do not drift in v0 (06-04 §3.2's twenty-year adjustment toward a
//! persistently favoured configuration is not built), and nobody sees another's.

use civ_core::PermanentId;
use civ_core::rng::Rng64;

use crate::demography::normal;

/// A value (content kind `value`, content API 43). Every number is a design prior.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueDef {
    pub id: String,
    pub name: String,
    pub description: String,
    /// What holding it more or less than most means, in words: "holds safety from want and harm
    /// dear", "cares little for safety from want and harm".
    pub high: String,
    pub low: String,
    /// The mean and spread of a founder's value before it is squashed into −1 to 1 (tanh), and
    /// how much of its parents' mean a child takes on.
    pub mean: f64,
    pub sd: f64,
    pub heritability: f64,
    /// Points a law that bears fully on it adds for one who holds it fully.
    pub weight: f64,
}

/// What one person holds of one value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Held {
    pub holder: PermanentId,
    /// The value, by index in the catalog's values.
    pub value: u16,
    /// −1 (less than most) to 1 (more than most).
    pub v: f32,
}

/// What everyone holds, in (holder, value) order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Values {
    pub held: Vec<Held>,
}

impl Values {
    fn find(&self, holder: PermanentId, value: u16) -> Result<usize, usize> {
        self.held
            .binary_search_by(|h| (h.holder, h.value).cmp(&(holder, value)))
    }

    /// What `holder` holds of value `value`, if they hold it yet.
    pub fn get(&self, holder: PermanentId, value: u16) -> Option<f32> {
        self.find(holder, value).ok().map(|i| self.held[i].v)
    }

    /// Holds `h`, in its place.
    pub fn insert(&mut self, h: Held) {
        match self.find(h.holder, h.value) {
            Ok(i) => self.held[i] = h,
            Err(i) => self.held.insert(i, h),
        }
    }

    /// What `holder` holds, in value order.
    pub fn held_by(&self, holder: PermanentId) -> &[Held] {
        let from = self.held.partition_point(|h| h.holder < holder);
        let to = self.held.partition_point(|h| h.holder <= holder);
        &self.held[from..to]
    }

    /// The points what `holder` holds adds to what they make of a law that bears `bears` (value
    /// index, how far it bears, −1 to 1) on the values `defs` names.
    pub fn points(&self, holder: PermanentId, bears: &[(u16, f32)], defs: &[ValueDef]) -> f64 {
        bears
            .iter()
            .filter_map(|&(k, b)| {
                let v = self.get(holder, k)?;
                let w = defs.get(usize::from(k))?.weight;
                Some(w * f64::from(v) * f64::from(b))
            })
            .sum()
    }
}

/// A value of `def`, −1 to 1, drawn from `rng`: a founder's from the content's spread, a child's
/// pulled toward the mean of its parents' (as [`crate::crime::draw_objection`] is).
pub fn draw(mother: Option<f32>, father: Option<f32>, def: &ValueDef, rng: &mut Rng64) -> f32 {
    let sd = def.sd.max(1e-6);
    let standard = |v: f32| (f64::from(v).clamp(-0.999_999, 0.999_999).atanh() - def.mean) / sd;
    let parents: Vec<f64> = [mother, father]
        .into_iter()
        .flatten()
        .map(standard)
        .collect();
    let z = if parents.is_empty() {
        normal(rng)
    } else {
        let h = def.heritability.clamp(0.0, 1.0);
        let mid = parents.iter().sum::<f64>() / parents.len() as f64;
        h * mid + (1.0 - h * h / 2.0).max(0.0).sqrt() * normal(rng)
    };
    (def.mean + sd * z).tanh() as f32
}

/// What someone holds of a value, in words: the content's `high` or `low`, or "cares as most do
/// for ..." near the middle.
pub fn words(v: f64, def: &ValueDef) -> String {
    match v {
        v if v >= 0.25 => def.high.clone(),
        v if v <= -0.25 => def.low.clone(),
        _ => format!("cares as most do for {}", def.name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn safety() -> ValueDef {
        ValueDef {
            id: "core:value/security".to_owned(),
            name: "safety from want and harm".to_owned(),
            description: String::new(),
            high: "holds safety from want and harm dear".to_owned(),
            low: "cares little for safety from want and harm".to_owned(),
            mean: 0.0,
            sd: 0.8,
            heritability: 0.5,
            weight: 1.0,
        }
    }

    #[test]
    fn values_spread_about_the_middle_and_children_take_after_their_parents() {
        let d = safety();
        let mut rng = Rng64::seed_from_u64(11);
        let n = 4000;
        let founders: Vec<f32> = (0..n).map(|_| draw(None, None, &d, &mut rng)).collect();
        let mean = founders.iter().map(|&v| f64::from(v)).sum::<f64>() / f64::from(n);
        assert!(mean.abs() < 0.05, "{mean}");
        assert!(founders.iter().all(|v| v.abs() < 1.0));
        assert!(founders.iter().filter(|v| v.abs() > 0.5).count() > n as usize / 5);
        let kids: f64 = (0..n)
            .map(|_| f64::from(draw(Some(0.8), Some(0.8), &d, &mut rng)))
            .sum::<f64>()
            / f64::from(n);
        assert!(kids > 0.3, "{kids}");
    }

    #[test]
    fn a_law_weighs_what_it_bears_on_by_what_each_holds() {
        let id = |n| PermanentId::from_raw(n).expect("nonzero");
        let defs = [
            safety(),
            ValueDef {
                weight: 2.0,
                ..safety()
            },
        ];
        let mut vs = Values::default();
        for (h, k, v) in [(2, 1, -0.5), (1, 0, 0.5), (2, 0, 1.0)] {
            vs.insert(Held {
                holder: id(h),
                value: k,
                v,
            });
        }
        assert_eq!(vs.held_by(id(2)).len(), 2);
        // Bears fully on the first, against the second.
        let bears = [(0, 1.0), (1, -1.0)];
        assert!((vs.points(id(2), &bears, &defs) - (1.0 + 1.0)).abs() < 1e-6);
        assert!(
            (vs.points(id(1), &bears, &defs) - 0.5).abs() < 1e-6,
            "holds only the first"
        );
        assert_eq!(vs.points(id(3), &bears, &defs), 0.0, "holds nothing yet");
        assert_eq!(words(0.6, &defs[0]), "holds safety from want and harm dear");
        assert_eq!(
            words(0.0, &defs[0]),
            "cares as most do for safety from want and harm"
        );
    }
}
