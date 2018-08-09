use std::cmp;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::{Europe::Helsinki, Tz};

use model::Model;

pub struct Measurements(pub Vec<Measurement>);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Resolution {
    Hour,
    Day,
    Month,
    Year,
}

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
    /// Local (Finnish) time
    pub localtime: DateTime<Tz>,
    /// Electricity consumption in kWh
    pub consumption: f64,
    /// Quality of electricity (probably percent)
    pub quality: i8,
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
        let mut measurements = Vec::new();

        for resolution in &[
            Resolution::Hour,
            Resolution::Day,
            Resolution::Month,
            Resolution::Year,
        ] {
            let mut meas = convert_one_resolution(*resolution, model);
            measurements.append(&mut meas.0);
        }

        measurements.sort();
        Measurements(measurements)
    }
}

fn convert_one_resolution(resolution: Resolution, model: &Model) -> Measurements {
    let mut measurements = Vec::new();

    let root = match resolution {
        Resolution::Hour => &model.hours,
        Resolution::Day => &model.days,
        Resolution::Month => &model.months,
        Resolution::Year => &model.years,
    };

    let consumptions = &root.consumptions;
    let statuses = &root.consumption_statuses.data;
    let temps = &root.temperature.data;

    for consumptions in consumptions {
        let tariff = match &*consumptions.tariff_time_zone_name {
            "Päivä" => Tariff::Day,
            "Yö" => Tariff::Night,
            name => panic!("Unknown tariff encountered: {}", name),
        };

        for data in &consumptions.series.data {
            let naive_date = NaiveDateTime::from_timestamp((data[0] / 1000.0) as i64, 0);
            let localtime = Helsinki.from_local_datetime(&naive_date).unwrap();
            let timestamp: DateTime<Utc> = localtime.with_timezone(&Utc);

            let consumption = data[1];
            let mut quality: i8 = -1;
            let mut temperature = ::std::f64::NAN;

            for status in statuses {
                if (status[0] - data[0]).abs() < 1.0 {
                    quality = status[1] as i8;
                    break;
                }
            }

            for temp in temps {
                if (temp[0] - data[0]).abs() < 1.0 {
                    temperature = temp[1];
                    break;
                }
            }

            let price = {
                let p = find_price(timestamp, tariff, model);
                let energy = p.energy.map(|p| p * consumption);
                let transfer = p.transfer.map(|p| p * consumption);
                Price { energy, transfer }
            };

            let meas = Measurement {
                timestamp,
                localtime,
                consumption,
                quality,
                temperature,
                tariff,
                resolution,
                price,
            };
            measurements.push(meas);
        }
    }

    measurements.sort();
    Measurements(measurements)
}

fn find_price(timestamp: DateTime<Utc>, tariff: Tariff, model: &Model) -> Price {
    let (transfer_list, energy_list) = match tariff {
        Tariff::Day => (
            &model.network_price_list.time_based_energy_day_prices,
            &model.sales_price_list.time_based_energy_day_prices,
        ),
        Tariff::Night => (
            &model.network_price_list.time_based_energy_night_prices,
            &model.sales_price_list.time_based_energy_night_prices,
        ),
    };

    let mut transfer = None;
    for t in transfer_list {
        if timestamp >= t.start_time && timestamp <= t.end_time {
            transfer = Some(t.price_with_vat);
            break;
        }
    }

    let mut energy = None;
    for e in energy_list {
        if timestamp >= e.start_time && timestamp <= e.end_time {
            energy = Some(e.price_with_vat);
            break;
        }
    }

    Price { transfer, energy }
}
