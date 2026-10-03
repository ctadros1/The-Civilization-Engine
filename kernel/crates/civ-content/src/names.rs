//! Name lists (`kind = "names"`): given names by sex, and parts for place names (research 06-07).

use civ_agents::params::NameParams;
use serde::Deserialize;

/// The `kind` value of a name list.
pub const KIND: &str = "names";
/// The id segment: `pack:names/name`.
pub const ID_KIND: &str = "names";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NamesFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub male: Vec<String>,
    pub female: Vec<String>,
    pub place_first: Vec<String>,
    pub place_second: Vec<String>,
}

impl NamesFile {
    /// The parameters.
    pub fn params(&self) -> NameParams {
        NameParams {
            male: self.male.clone(),
            female: self.female.clone(),
            place_first: self.place_first.clone(),
            place_second: self.place_second.clone(),
        }
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        if self.name.trim().is_empty() {
            p.push("a name list needs a display `name`".to_owned());
        }
        for (field, list) in [
            ("male", &self.male),
            ("female", &self.female),
            ("place_first", &self.place_first),
            ("place_second", &self.place_second),
        ] {
            if list.is_empty() {
                p.push(format!("`{field}` needs at least one name"));
            }
            if list.iter().any(|n| n.trim().is_empty() || n.trim() != n) {
                p.push(format!(
                    "`{field}` has an empty name or one with spaces around it"
                ));
            }
        }
        p
    }
}
