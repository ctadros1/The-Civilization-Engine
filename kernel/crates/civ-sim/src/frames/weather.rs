//! The weather on the wire (wire 1.24, M3c slice U, ADR-0012): today's on the clock, in figures
//! and words, and the record of every month for the weather panel.

use civ_land::weather::{SNOW_COVER_MM, WeatherDay, month_of};
use civ_land::{Weather, WeatherParams};
use civ_schema::flatbuffers::{FlatBufferBuilder, WIPOffset};
use civ_schema::wire;

use crate::Sim;
use crate::frames::response;

/// Changes each day lived (0 = no world): fetch the weather's record when it differs.
pub fn weather_rev(sim: &Sim) -> u64 {
    u64::try_from(sim.land.weather.today.day + 1).unwrap_or(0)
}

/// How much fell today, in words: "dry", "a trace of rain", "light snow", "heavy rain".
fn fall(params: &WeatherParams, mean_c: f64, precip_mm: f64) -> &'static str {
    let snow = mean_c < params.snow_below_c;
    match (precip_mm, snow) {
        (p, _) if p < 0.2 => "dry",
        (p, false) if p < params.wet_day_mm => "a trace of rain",
        (p, true) if p < params.wet_day_mm => "a few flakes of snow",
        (p, false) if p < 5.0 => "light rain",
        (p, true) if p < 5.0 => "light snow",
        (p, false) if p < 15.0 => "rain",
        (p, true) if p < 15.0 => "snow",
        (_, false) => "heavy rain",
        (_, true) => "heavy snow",
    }
}

/// Today's weather on the valley floor in words, as people would say it: "6 °C, light rain",
/// "−3 °C, snow; snow lying", "21 °C, dry; frost at night; dry ground", "14 °C, rain on dry
/// ground".
pub fn weather_words(params: &WeatherParams, w: &Weather) -> String {
    let t = &w.today;
    let mean = f64::from(t.mean_c);
    let fell = fall(params, mean, f64::from(t.precip_mm));
    let soil = w.soil_mm / params.soil_water_mm.max(1e-9);
    let ground = if soil < 0.2 {
        Some("parched ground")
    } else if soil < 1.0 - params.easy_water_share {
        Some("dry ground")
    } else {
        None
    };
    let mut s = match (fell, ground) {
        ("dry", _) | (_, None) => format!("{mean:.0} °C, {fell}"),
        (_, Some(g)) => format!("{mean:.0} °C, {fell} on {g}"),
    };
    if t.min_c < 0.0 && mean >= 0.0 {
        s.push_str("; frost at night");
    }
    if w.snow_at(params.normals_at_m) >= SNOW_COVER_MM {
        s.push_str("; snow lying");
    }
    if let (Some(g), "dry") = (ground, fell) {
        s.push_str(&format!("; {g}"));
    }
    s
}

/// The sixteen points of the compass, clockwise from north.
const COMPASS: [&str; 16] = [
    "north",
    "north-north-east",
    "north-east",
    "east-north-east",
    "east",
    "east-south-east",
    "south-east",
    "south-south-east",
    "south",
    "south-south-west",
    "south-west",
    "west-south-west",
    "west",
    "west-north-west",
    "north-west",
    "north-north-west",
];

/// The point of the compass nearest `deg` degrees clockwise from north.
pub fn compass(deg: f64) -> &'static str {
    let i = ((deg.rem_euclid(360.0) / 22.5).round() as usize) % 16;
    COMPASS[i]
}

/// A day's mean wind in words, as people would say it, by the Beaufort scale's bands: "calm",
/// "a light breeze", "a strong wind", "a gale".
pub fn wind_strength(ms: f64) -> &'static str {
    match ms {
        v if v < 0.5 => "calm",
        v if v < 1.6 => "light air",
        v if v < 3.4 => "a light breeze",
        v if v < 5.5 => "a gentle breeze",
        v if v < 8.0 => "a moderate wind",
        v if v < 10.8 => "a fresh wind",
        v if v < 13.9 => "a strong wind",
        v if v < 17.2 => "a near gale",
        v if v < 24.5 => "a gale",
        _ => "a storm",
    }
}

/// How dry exposed thatch is against the landscape's average day, in words.
pub fn dryness(moisture: f64, usual: f64) -> &'static str {
    match moisture {
        m if m >= 0.35 => "thatch wet through",
        m if m >= usual * 1.25 => "thatch damp",
        m if m <= usual * 0.5 => "thatch tinder-dry",
        m if m <= usual * 0.8 => "thatch drier than usual",
        _ => "thatch as dry as usual",
    }
}

/// Today's wind and dryness in words: "a gentle breeze from the west-south-west, gusts to 7 m/s;
/// thatch drier than usual", "calm; thatch damp" (M6b slice BC).
pub fn wind_words(t: &WeatherDay, usual: f64) -> String {
    let speed = f64::from(t.wind_ms);
    let wind = if speed < 0.5 {
        "calm".to_owned()
    } else {
        format!(
            "{} from the {}, gusts to {:.0} m/s",
            wind_strength(speed),
            compass(f64::from(t.wind_from_deg)),
            t.gust_ms
        )
    };
    format!("{wind}; {}", dryness(f64::from(t.fuel_moisture), usual))
}

/// Today's weather on the valley floor, for the clock.
pub fn day_weather<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    sim: &Sim,
) -> WIPOffset<wire::DayWeather<'a>> {
    let params = &sim.rules.land.weather;
    let w = &sim.land.weather;
    let words = fbb.create_string(&weather_words(params, w));
    let usual = sim.land.climatology.fuel_moisture_mean;
    let wind = fbb.create_string(&wind_words(&w.today, usual));
    let t = &w.today;
    wire::DayWeather::create(
        fbb,
        &wire::DayWeatherArgs {
            precip_mm: t.precip_mm,
            mean_c: t.mean_c,
            min_c: t.min_c,
            max_c: t.max_c,
            snow_mm: w.snow_at(params.normals_at_m) as f32,
            soil: (w.soil_mm / params.soil_water_mm.max(1e-9)) as f32,
            words: Some(words),
            snow_line_m: w.snow_line_m() as f32,
            wind_ms: t.wind_ms,
            wind_from_deg: t.wind_from_deg,
            gust_ms: t.gust_ms,
            fuel_moisture: t.fuel_moisture,
            fuel_usual: usual as f32,
            wind_words: Some(wind),
        },
    )
}

/// A `Response` with every month's weather on the valley floor, oldest first, beside what each
/// month usually brings.
pub fn weather_response(sim: &Sim) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let params = &sim.rules.land.weather;
    let land = &sim.land;
    let months: Vec<_> = land
        .weather
        .months
        .iter()
        .map(|m| {
            let i = usize::from(m.month.min(11));
            wire::WeatherMonthInfo::create(
                &mut fbb,
                &wire::WeatherMonthInfoArgs {
                    year: m.year,
                    month: m.month,
                    days: m.days,
                    precip_mm: m.precip_mm,
                    usual_mm: land.climatology.month_precip_mm[i] as f32,
                    wet_days: m.wet_days,
                    mean_c: m.mean_c() as f32,
                    usual_c: params.mean_c[i] as f32,
                    min_c: m.min_c,
                    max_c: m.max_c,
                    frost_days: m.frost_days,
                    snow_days: m.snow_days,
                    soil: m.soil() as f32,
                    held: m.held,
                },
            )
        })
        .collect();
    let months = fbb.create_vector(&months);
    let today = day_weather(&mut fbb, sim);
    // The observer's drought while it holds (wire 1.63).
    let drought = land.weather.hold.map(|(value, until)| {
        fbb.create_string(&format!(
            "The observer holds back the rain, {:.1} standard deviations dry, until {}",
            -value,
            civ_agents::polity::day_words(civ_core::SimTime::from_minutes(until * 1440))
        ))
    });
    let body = wire::WeatherReport::create(
        &mut fbb,
        &wire::WeatherReportArgs {
            rev: weather_rev(sim),
            height_m: params.normals_at_m as f32,
            annual_mm: land.climatology.annual_mm as f32,
            today: Some(today),
            months: Some(months),
            month: month_of(land.weather.today.day) as u8,
            drought,
        },
    );
    response(fbb, wire::ResponseBody::WeatherReport, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> WeatherParams {
        WeatherParams {
            wet_days: [0.4; 12],
            rain_share: [1.0 / 12.0; 12],
            persistence: 0.3,
            gamma_shape: 0.8,
            wet_day_mm: 1.0,
            mean_c: [10.0; 12],
            day_sd_c: [3.0; 12],
            day_persistence: 0.7,
            day_range_c: [10.0; 12],
            wet_day_range: 0.75,
            wet_day_cooling_c: [0.0; 12],
            normals_at_m: 150.0,
            lapse_c_per_km: 6.5,
            slow_months: 12.0,
            slow_amount: 0.1,
            slow_wet_days: 0.1,
            slow_warmth_c: [0.0; 12],
            snow_below_c: 0.5,
            melt_mm_per_c: 3.0,
            soil_water_mm: 100.0,
            easy_water_share: 0.55,
            cover_kc: 0.9,
            wet_ground_mm: 5.0,
            frozen_below_c: 0.0,
            storm_median_pa: 250.0,
            storm_spread: 0.6,
            roof_snow_share: 0.8,
            roof_snow_full_deg: 30.0,
            roof_snow_shed_deg: 60.0,
            wind_from_deg: 250.0,
            wind_mean_ms: 2.0,
            wind_sd_ms: 2.5,
            wind_persistence: 0.6,
            gust_factor: 1.7,
            storm_veer_deg: 30.0,
            fuel_dry_moisture: [0.25, 0.08],
            fuel_days: 3.0,
            fuel_wetting_per_mm: 0.05,
            fuel_wet_moisture: 0.6,
        }
    }

    fn weather(mean: f32, precip: f32, min: f32, soil_mm: f64, snow: f32) -> Weather {
        Weather {
            today: civ_land::WeatherDay {
                day: 100,
                wet: precip > 0.0,
                precip_mm: precip,
                mean_c: mean,
                min_c: min,
                max_c: mean + 5.0,
                ..civ_land::WeatherDay::default()
            },
            anomaly: 0.0,
            slow: 0.0,
            wind: (0.0, 0.0),
            snow_base_m: 100.0,
            snow_mm: vec![snow],
            soil_mm,
            cover_ease: 1.0,
            months: Vec::new(),
            hold: None,
        }
    }

    #[test]
    fn the_wind_and_the_thatch_are_told_in_words() {
        let day = |ms: f32, from: f32, fuel: f32| civ_land::WeatherDay {
            wind_ms: ms,
            wind_from_deg: from,
            gust_ms: ms * 1.7,
            fuel_moisture: fuel,
            ..civ_land::WeatherDay::default()
        };
        assert_eq!(
            wind_words(&day(4.0, 250.0, 0.14), 0.15),
            "a gentle breeze from the west-south-west, gusts to 7 m/s; thatch as dry as usual"
        );
        assert_eq!(
            wind_words(&day(0.2, 90.0, 0.4), 0.15),
            "calm; thatch wet through"
        );
        assert_eq!(
            wind_words(&day(19.0, 2.0, 0.06), 0.15),
            "a gale from the north, gusts to 32 m/s; thatch tinder-dry"
        );
        assert_eq!(compass(359.0), "north");
        assert_eq!(compass(135.0), "south-east");
        assert_eq!(compass(-90.0), "west");
        assert_eq!(dryness(0.2, 0.15), "thatch damp");
        assert_eq!(dryness(0.11, 0.15), "thatch drier than usual");
    }

    #[test]
    fn the_day_is_told_in_words() {
        let p = params();
        assert_eq!(
            weather_words(&p, &weather(6.0, 3.0, 2.0, 80.0, 0.0)),
            "6 °C, light rain"
        );
        assert_eq!(
            weather_words(&p, &weather(14.0, 9.0, 8.0, 30.0, 0.0)),
            "14 °C, rain on dry ground"
        );
        assert_eq!(
            weather_words(&p, &weather(21.0, 0.0, -1.0, 10.0, 0.0)),
            "21 °C, dry; frost at night; parched ground"
        );
        assert_eq!(
            weather_words(&p, &weather(-3.0, 9.0, -6.0, 100.0, 20.0)),
            "-3 °C, snow; snow lying"
        );
        assert_eq!(fall(&p, 6.0, 0.0), "dry");
        assert_eq!(fall(&p, 6.0, 0.5), "a trace of rain");
        assert_eq!(fall(&p, 6.0, 3.0), "light rain");
        assert_eq!(fall(&p, 6.0, 9.0), "rain");
        assert_eq!(fall(&p, 6.0, 30.0), "heavy rain");
        assert_eq!(fall(&p, -3.0, 9.0), "snow");
        assert_eq!(fall(&p, -3.0, 0.5), "a few flakes of snow");
    }
}
