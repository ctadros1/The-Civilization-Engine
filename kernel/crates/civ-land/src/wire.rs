//! The wire (M11a, third part; ADR-0036 §3–§4): lines of current from a station, in a radial tree,
//! and the buildings hooked up to them; an hour's dispatch shares a station's output down its tree,
//! each line passing what its rating allows less its I²R losses, hookups served in the order made
//! and one left unserved keeping its cause (research 12-09 §5.2–§5.4; 07-13 §1.3). Pure functions
//! of their inputs; lines and hookups are saved with the land (schema 220), and nothing lays or
//! dispatches them in a lived day yet.

use civ_core::PermanentId;

/// The world's direct current and the line it is carried on (M11a, third part; ADR-0036 §3).
/// Authored in the land profile's `[wire]` (content API 202).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WireParams {
    /// The one voltage of the world's direct current, volts.
    pub voltage_v: f64,
    /// Resistance of a metre of line, out and back, ohms.
    pub ohm_per_m: f64,
    /// The good its conductor is made of (an index into the catalog's goods).
    pub good: usize,
    /// Kilograms of that good a metre of line takes, out and back.
    pub good_kg_m: f64,
    /// Hours of a capable adult's work to set poles and string a metre of line.
    pub lay_h_m: f64,
    /// The most a line passes, kW.
    pub rating_kw: f64,
    /// The years a line laid serves.
    pub life_years: f64,
}

/// A line: the station's holder whose current it carries, the line it hangs from (`None` for one
/// leaving the station), the cells it runs along from its start, the most it passes, kW, its
/// length and how much of it is laid, metres, and whether it is broken. It carries only once
/// whole.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub id: PermanentId,
    pub station: PermanentId,
    pub parent: Option<PermanentId>,
    pub cells: Vec<u32>,
    pub rating_kw: f64,
    pub length_m: f64,
    pub laid_m: f64,
    pub broken: bool,
}

impl Line {
    /// Whether its whole length is laid.
    pub fn whole(&self) -> bool {
        self.laid_m + 1e-3 >= self.length_m
    }

    /// The kilowatts lost carrying `kw` along it at `p`'s voltage: I²R with I = P / V.
    pub fn loss_kw(&self, kw: f64, p: &WireParams) -> f64 {
        let amps = kw * 1000.0 / p.voltage_v.max(1e-9);
        amps * amps * p.ohm_per_m * self.length_m / 1000.0
    }
}

/// Why a hookup was or was not served in an hour (12-09 §1.5: causes kept apart).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Cause {
    /// Served.
    #[default]
    Served,
    /// The station gave too little for it and its line's losses.
    StationShort,
    /// A line on its way from the station could not pass it as well as those ahead of it.
    LineFull,
    /// A line on its way is broken or not yet whole.
    LineDown,
    /// Its account is cut off.
    CutOff,
}

impl Cause {
    /// As saved.
    pub fn code(self) -> u8 {
        match self {
            Cause::Served => 0,
            Cause::StationShort => 1,
            Cause::LineFull => 2,
            Cause::LineDown => 3,
            Cause::CutOff => 4,
        }
    }

    /// From its saved code; an unknown one reads as served.
    pub fn from_code(code: u8) -> Cause {
        match code {
            1 => Cause::StationShort,
            2 => Cause::LineFull,
            3 => Cause::LineDown,
            4 => Cause::CutOff,
            _ => Cause::Served,
        }
    }
}

/// A building hooked up to a line, the household that keeps it, the kilowatts it asks in the hour,
/// whether its account is cut off, and its last evening: the minute it is lit until (-1 never)
/// and why it was or was not served.
#[derive(Clone, Debug, PartialEq)]
pub struct Hookup {
    pub building: PermanentId,
    pub household: PermanentId,
    pub line: PermanentId,
    pub ask_kw: f64,
    pub cut_off: bool,
    pub lit_until: i64,
    pub cause: Cause,
}

/// Every line and hookup, in the order made (saved in the `wires` section, schema 220).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Wires {
    pub lines: Vec<Line>,
    pub hookups: Vec<Hookup>,
}

impl Wires {
    /// Whether a hookup of `household` is lit by the wire at `minute`.
    pub fn lit(&self, household: PermanentId, minute: i64) -> bool {
        self.lit_until(household, minute).is_some()
    }

    /// The minute a hookup of `household` served at `minute` is lit until, if any is.
    pub fn lit_until(&self, household: PermanentId, minute: i64) -> Option<i64> {
        self.hookups
            .iter()
            .filter(|h| {
                h.household == household && h.cause == Cause::Served && minute < h.lit_until
            })
            .map(|h| h.lit_until)
            .max()
    }
}

/// One hour's dispatch from station `station` giving `output_kw` (ADR-0036 §4): each hookup on its
/// tree in the order given is served whole or not at all, where every line from it up to the
/// station is whole and unbroken and can pass the flow it would then carry, and the station covers
/// what it then delivers and every line's losses. Returns each hookup's cause in the order given
/// (hookups on other stations' lines read as `LineDown`) and the kilowatts the station sends out,
/// losses included. A line whose parent is missing is down; a parent loop ends the climb.
pub fn dispatch(
    lines: &[Line],
    hookups: &[Hookup],
    station: PermanentId,
    output_kw: f64,
    p: &WireParams,
) -> (Vec<Cause>, f64) {
    let index = |id: PermanentId| {
        lines
            .iter()
            .position(|l| l.id == id && l.station == station)
    };
    // The lines from a hookup's own up to the station, or `None` where one is missing.
    let path = |first: PermanentId| -> Option<Vec<usize>> {
        let mut out = Vec::new();
        let mut at = Some(first);
        while let Some(id) = at {
            let i = index(id)?;
            if out.contains(&i) || out.len() > lines.len() {
                break;
            }
            out.push(i);
            at = lines[i].parent;
        }
        Some(out)
    };
    let mut flow = vec![0.0f64; lines.len()];
    let sent = |flow: &[f64]| -> f64 {
        let delivered: f64 = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.parent.is_none())
            .map(|(i, _)| flow[i])
            .sum();
        delivered
            + lines
                .iter()
                .enumerate()
                .map(|(i, l)| l.loss_kw(flow[i], p))
                .sum::<f64>()
    };
    let mut causes = Vec::with_capacity(hookups.len());
    for h in hookups {
        let Some(way) = path(h.line) else {
            causes.push(Cause::LineDown);
            continue;
        };
        if h.cut_off {
            causes.push(Cause::CutOff);
            continue;
        }
        if way.iter().any(|&i| lines[i].broken || !lines[i].whole()) {
            causes.push(Cause::LineDown);
            continue;
        }
        // A line carries what passes beyond it and its own losses below it are counted at the
        // station; its rating bounds the flow into it.
        let mut next = flow.clone();
        for &i in &way {
            next[i] += h.ask_kw.max(0.0);
        }
        if way.iter().any(|&i| next[i] > lines[i].rating_kw + 1e-9) {
            causes.push(Cause::LineFull);
            continue;
        }
        if sent(&next) > output_kw + 1e-9 {
            causes.push(Cause::StationShort);
            continue;
        }
        flow = next;
        causes.push(Cause::Served);
    }
    (causes, sent(&flow))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("id")
    }

    const P: WireParams = WireParams {
        voltage_v: 110.0,
        ohm_per_m: 0.0005,
        good: 0,
        good_kg_m: 0.6,
        lay_h_m: 0.5,
        rating_kw: 6.0,
        life_years: 40.0,
    };

    fn line(n: u64, parent: Option<u64>, rating_kw: f64, length_m: f64) -> Line {
        Line {
            id: id(n),
            station: id(1),
            parent: parent.map(id),
            cells: Vec::new(),
            rating_kw,
            length_m,
            laid_m: length_m,
            broken: false,
        }
    }

    fn hookup(building: u64, line: u64, ask_kw: f64) -> Hookup {
        Hookup {
            building: id(building),
            household: id(building),
            line: id(line),
            ask_kw,
            cut_off: false,
            lit_until: -1,
            cause: Cause::Served,
        }
    }

    #[test]
    fn a_line_loses_the_square_of_its_current_times_its_resistance() {
        // 5 kW at 110 V is 45.5 A; 200 m at 0.0005 Ω a metre is 0.1 Ω: 207 W lost.
        let l = line(10, None, 10.0, 200.0);
        assert!((l.loss_kw(5.0, &P) - 0.2066).abs() < 1e-3);
        // Twice the power, four times the loss.
        assert!((l.loss_kw(10.0, &P) - 4.0 * l.loss_kw(5.0, &P)).abs() < 1e-9);
    }

    #[test]
    fn hookups_are_served_in_order_while_lines_and_the_station_can_carry_them() {
        // A trunk of 6 kW from the station, a branch of 2 kW hanging from it.
        let lines = vec![line(10, None, 6.0, 100.0), line(11, Some(10), 2.0, 100.0)];
        let hookups = vec![
            hookup(100, 11, 1.5),
            // The branch is full after the first.
            hookup(101, 11, 1.0),
            hookup(102, 10, 3.0),
            // The trunk would pass 7.5 kW.
            hookup(103, 10, 3.0),
        ];
        let (causes, sent) = dispatch(&lines, &hookups, id(1), 100.0, &P);
        assert_eq!(
            causes,
            vec![
                Cause::Served,
                Cause::LineFull,
                Cause::Served,
                Cause::LineFull
            ]
        );
        assert!(
            sent > 4.5 && sent < 4.7,
            "4.5 kW delivered and losses: {sent}"
        );
        // The same with a station of 4.55 kW: the third (4.5 kW and 0.09 of losses) no longer fits.
        let (causes, _) = dispatch(&lines, &hookups, id(1), 4.55, &P);
        assert_eq!(causes[2], Cause::StationShort);
    }

    #[test]
    fn a_hookup_beyond_a_line_down_or_on_another_station_or_cut_off_is_not_served() {
        let mut lines = vec![line(10, None, 6.0, 100.0), line(11, Some(10), 2.0, 100.0)];
        lines[0].laid_m = 50.0;
        let mut cut = hookup(102, 10, 1.0);
        cut.cut_off = true;
        let hookups = vec![hookup(100, 11, 1.0), hookup(101, 99, 1.0), cut];
        let (causes, sent) = dispatch(&lines, &hookups, id(1), 100.0, &P);
        assert_eq!(
            causes,
            vec![Cause::LineDown, Cause::LineDown, Cause::CutOff]
        );
        assert_eq!(sent, 0.0);
        // Another station's tree reads as down from this one.
        let (causes, _) = dispatch(&lines, &[hookup(100, 10, 1.0)], id(2), 100.0, &P);
        assert_eq!(causes, vec![Cause::LineDown]);
    }
}
