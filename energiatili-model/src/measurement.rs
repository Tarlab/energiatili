use std::cmp;
use std::collections::{BTreeMap, BTreeSet};

use chrono::offset::LocalResult;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Helsinki;
use num_traits::{cast, NumCast};
use rayon::prelude::*;
use serde_json::Value;

use crate::model::Model;

pub struct Measurements(pub BTreeSet<Measurement>);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Resolution {
    Hour,
    Day,
    Month,
    Year,
}

pub static RESOLUTIONS: &[Resolution] = &[
    Resolution::Hour,
    Resolution::Day,
    Resolution::Month,
    Resolution::Year,
];

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Tariff {
    Day,
    Night,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Price {
    pub transfer: Option<f64>,
    pub energy: Option<f64>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Measurement {
    /// Time of measurement
    pub timestamp: DateTime<Utc>,
    /// Electricity consumption in kWh
    pub consumption: f64,
    /// Quality of electricity (probably percent)
    pub quality: u8,
    /// Outside temperature (in °C)
    pub temperature: f64,
    /// Day or Night pricing
    pub tariff: Tariff,
    /// Time resolution
    pub resolution: Resolution,
    /// Price for consumption
    pub price: Price,
}

impl Eq for Measurement {}

impl Ord for Measurement {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl<'a> From<&'a Model> for Measurements {
    fn from(model: &Model) -> Self {
        let measurements: BTreeSet<Measurement> = RESOLUTIONS
            .into_par_iter()
            .map(|resolution| convert_one_resolution(*resolution, model).0)
            .flatten()
            .collect();

        Measurements(measurements)
    }
}

fn convert_one_resolution(resolution: Resolution, model: &Model) -> Measurements {
    let root = match resolution {
        Resolution::Hour => &model.hours,
        Resolution::Day => &model.days,
        Resolution::Month => &model.months,
        Resolution::Year => &model.years,
    };

    let status_map: BTreeMap<i64, u8> = values_into_map(&root.consumption_statuses.data);
    let temperature_map: BTreeMap<i64, f64> = values_into_map(&root.temperature.data);

    let measurements = root
        .consumptions
        .par_iter()
        .map(|consumptions| {
            let consumption_map: BTreeMap<i64, f64> = values_into_map(&consumptions.series.data);
            let tariff = match &*consumptions.tariff_time_zone_name {
                "Päivä" => Tariff::Day,
                "Yö" => Tariff::Night,
                name => panic!("Unknown tariff encountered: {}", name),
            };
            consumption_map
                .into_iter()
                .map(|(ts, consumption)| {
                    build_measurement(
                        ts,
                        consumption,
                        &status_map,
                        &temperature_map,
                        tariff,
                        model,
                        resolution,
                    )
                })
                .collect::<BTreeSet<_>>()
        })
        .flatten()
        .collect();

    Measurements(measurements)
}

fn build_measurement(
    ts: i64,
    consumption: f64,
    status_map: &BTreeMap<i64, u8>,
    temperature_map: &BTreeMap<i64, f64>,
    tariff: Tariff,
    model: &Model,
    resolution: Resolution,
) -> Measurement {
    let timestamp: DateTime<Utc> = convert_timestamp(ts);
    let quality = *status_map.get(&ts).unwrap_or(&0);
    let temperature = *temperature_map.get(&ts).unwrap_or(&::std::f64::NAN);

    let price = {
        let p = find_price(timestamp, tariff, model);
        let energy = p.energy.map(|p| p * consumption);
        let transfer = p.transfer.map(|p| p * consumption);
        Price { energy, transfer }
    };

    Measurement {
        timestamp,
        consumption,
        quality,
        temperature,
        tariff,
        resolution,
        price,
    }
}

fn find_price(timestamp: DateTime<Utc>, tariff: Tariff, model: &Model) -> Price {
    let transfer_list = match tariff {
        Tariff::Day => &model.network_price_list.time_based_energy_day_prices,
        Tariff::Night => &model.network_price_list.time_based_energy_night_prices,
    };

    let mut transfer = None;
    for t in transfer_list {
        if timestamp >= t.start_time && timestamp <= t.end_time {
            transfer = Some(t.price_with_vat);
            break;
        }
    }

    let mut energy = None;
    if let Some(energy_list) = &model.sales_price_list {
        let energy_list = match tariff {
            Tariff::Day => &energy_list.time_based_energy_day_prices,
            Tariff::Night => &energy_list.time_based_energy_night_prices,
        };

        for e in energy_list {
            if timestamp >= e.start_time && timestamp <= e.end_time {
                energy = Some(e.price_with_vat);
                break;
            }
        }
    }

    Price { transfer, energy }
}

fn values_into_map<T>(values: &[Value]) -> BTreeMap<i64, T>
where
    T: NumCast,
{
    values
        .iter()
        .map(|value| {
            let ts = value.get(0).and_then(Value::as_i64).expect("into_map ts");
            let val = value
                .get(1)
                .and_then(Value::as_f64)
                .and_then(cast)
                .expect("into_map value");
            (ts, val)
        })
        .collect()
}

fn convert_timestamp(timestamp: i64) -> DateTime<Utc> {
    let naive_date = NaiveDateTime::from_timestamp((timestamp / 1000) as i64, 0);
    let localtime = match Helsinki.from_local_datetime(&naive_date) {
        LocalResult::None => panic!("Couldn't convert local time"),
        LocalResult::Single(t) => t,
        LocalResult::Ambiguous(t, _) => t,
    };
    localtime.with_timezone(&Utc)
}
