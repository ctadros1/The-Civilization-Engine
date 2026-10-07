//! Ties and standing for the observer (wire 1.26, ADR-0014): a person's strongest ties with
//! their reasons in words, and each settlement's standing with its notables. The kernel renders
//! the words; the observer only shows them.

use civ_agents::Person;
use civ_agents::standing::Standing;
use civ_agents::ties::{DOMAINS, Domain};
use civ_core::PermanentId;
use civ_schema::flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector, WIPOffset};
use civ_schema::wire;

use super::response;
use crate::Sim;

/// The most ties the inspector is sent for one person.
pub const MAX_TIES_SHOWN: usize = 12;
/// The most rows a settlement's standing sends besides its notables.
pub const MAX_STANDING_ROWS: usize = 20;

/// `p`'s strongest ties, the most salient first, at most [`MAX_TIES_SHOWN`].
pub fn tie_lines<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    p: &Person,
) -> WIPOffset<Vector<'a, ForwardsUOffset<wire::TieLine<'a>>>> {
    let (pop, tp) = (&sim.people, &sim.rules.people.ties);
    let day = sim.now().day_index();
    let mut ties: Vec<_> = pop.ties.of(p.id).iter().map(|t| t.at(day, tp)).collect();
    ties.sort_by(|a, b| {
        b.salience(tp)
            .total_cmp(&a.salience(tp))
            .then(a.to.cmp(&b.to))
    });
    let lines: Vec<_> = ties
        .iter()
        .take(MAX_TIES_SHOWN)
        .map(|t| {
            let name = fbb.create_string(&pop.name_of(t.to));
            let esteem: [f32; DOMAINS] = Domain::ALL.map(|d| t.esteem(d, tp) as f32);
            let esteem = fbb.create_vector(&esteem);
            let reason = t.reason.map(|r| fbb.create_string(&r.words()));
            wire::TieLine::create(
                fbb,
                &wire::TieLineArgs {
                    person: t.to.get(),
                    name: Some(name),
                    familiarity: t.familiarity,
                    warmth: t.warmth,
                    esteem: Some(esteem),
                    help_h: t.help_h,
                    reason,
                    mutual: pop.ties.tie(t.to, p.id, day, tp).is_some(),
                },
            )
        })
        .collect();
    fbb.create_vector(&lines)
}

/// One adult's standing line.
pub fn standing_line<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
    r: &Standing,
) -> WIPOffset<wire::StandingLine<'a>> {
    let pop = &sim.people;
    let name = fbb.create_string(&pop.name_of(r.person));
    let esteem = fbb.create_vector(&r.esteem);
    wire::StandingLine::create(
        fbb,
        &wire::StandingLineArgs {
            person: r.person.get(),
            name: Some(name),
            household: pop.person(r.person).map_or(0, |p| p.household.get()),
            esteem: Some(esteem),
            influence: r.influence,
            notable: r.notable,
        },
    )
}

/// The rows of a settlement's standing the observer is sent: its notables, then the most
/// influential and esteemed, at most [`MAX_STANDING_ROWS`] besides the notables.
pub fn shown_rows(sim: &Sim, settlement: PermanentId) -> Vec<Standing> {
    let mut rows: Vec<Standing> = sim
        .people
        .standing
        .in_settlement(settlement)
        .copied()
        .collect();
    rows.sort_by(|a, b| {
        b.notable
            .cmp(&a.notable)
            .then(b.influence.cmp(&a.influence))
            .then(b.total().total_cmp(&a.total()))
            .then(a.person.cmp(&b.person))
    });
    let notables = rows.iter().filter(|r| r.notable).count();
    rows.truncate(notables + MAX_STANDING_ROWS);
    rows
}

/// A `Response` with every settlement's standing.
pub fn standing_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let table = &sim.people.standing;
    let domains: Vec<_> = Domain::ALL
        .iter()
        .map(|d| fbb.create_string(d.name()))
        .collect();
    let domains = fbb.create_vector(&domains);
    let mut list = Vec::new();
    for s in &sim.land.settlements {
        let adults = table.in_settlement(s.id).count() as u32;
        let rows: Vec<_> = shown_rows(sim, s.id)
            .iter()
            .map(|r| standing_line(&mut fbb, sim, r))
            .collect();
        let rows = fbb.create_vector(&rows);
        let name = fbb.create_string(&s.name);
        list.push(wire::SettlementStanding::create(
            &mut fbb,
            &wire::SettlementStandingArgs {
                settlement: s.id.get(),
                name: Some(name),
                adults,
                rows: Some(rows),
            },
        ));
    }
    let list = fbb.create_vector(&list);
    let minute = if table.day == i64::MIN {
        0
    } else {
        table.day * 1440
    };
    let body = wire::Standing::create(
        &mut fbb,
        &wire::StandingArgs {
            minute,
            domains: Some(domains),
            settlements: Some(list),
            ties: sim.people.ties.len() as u32,
            let_go: sim.people.ties.let_go,
        },
    );
    response(fbb, wire::ResponseBody::Standing, body)
}
