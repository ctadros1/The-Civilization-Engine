//! Waits worth telling (wire 1.85, M8b slice CN; ADR-0029 §9): a walker who stood a quarter of
//! an hour or more at a crossing or a gate, taking their turn.

use civ_agents::traffic::LongWait;

use super::{Page, Row, Step, WAIT};
use crate::Sim;
use crate::frames::traffic::facility_name;

pub fn wait_row(sim: &Sim, w: &LongWait) -> Row {
    let ahead = match w.ahead {
        0 => String::new(),
        1 => ", one walker ahead of them".to_owned(),
        n => format!(", {n} walkers ahead of them"),
    };
    Row {
        kind: WAIT,
        id: w.id,
        minute: w.time.minutes(),
        settlement: w.settlement,
        at: w.at,
        who: w.person,
        who_name: None,
        happened: format!(
            "waited {:.0} minutes to pass {}{ahead}",
            w.minutes,
            facility_name(sim, w.facility)
        ),
        known: "Their household remembers it".to_owned(),
        polity: "No gathering was told of it".to_owned(),
        observer: 0,
        outbreak: 0,
        conflagration: 0,
        flood: 0,
        storm: 0,
        open: false,
    }
}

pub fn wait_page(sim: &Sim, w: &LongWait) -> Page {
    let row = wait_row(sim, w);
    let place = facility_name(sim, w.facility);
    let steps = vec![
        Step::at(
            "Came",
            w.time.minutes(),
            format!("Reached {place} with {} ahead of them", w.ahead),
        ),
        Step::at(
            "Passed",
            w.time.minutes() + w.minutes.round() as i64,
            format!("Passed after {:.0} minutes", w.minutes),
        ),
    ];
    let truth = vec![format!(
        "{place} passes walkers one after another each way it allows; those who came before them passed first."
    )];
    let knowledge =
        vec!["Their household remembers the wait, fading as its other walks do.".to_owned()];
    let polity = vec!["No gathering was told of it.".to_owned()];
    (row, steps, Vec::new(), [truth, knowledge, polity])
}
