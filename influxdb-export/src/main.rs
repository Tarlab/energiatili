use std::io;

use influxdb;

use tokio::runtime::current_thread::Runtime;

use energiatili_model::measurement::{Measurements, Resolution, Tariff};
use energiatili_model::model::Model;

fn main() {
    env_logger::init();

    let stdin = io::stdin();
    let model = Model::from_reader(stdin).expect("read JSON Model");
    let measurements = Measurements::from(&model);

    let client = influxdb::Client::new("http://127.0.0.1:8086", "energiatili");

    let mut rt = Runtime::new().expect("Unable to create a runtime");

    for m in measurements.0 {
        let ts = m.timestamp.timestamp_nanos();
        let timestamp = influxdb::Timestamp::NANOSECONDS(ts as usize);
        let mut measurement = influxdb::Query::write_query(timestamp, "energiatili");

        measurement = measurement.add_field("consumption", m.consumption);
        measurement = measurement.add_field("quality", i64::from(m.quality));

        if let Some(price) = m.price.energy {
            measurement = measurement.add_field("energy_price", price);
        }

        if let Some(price) = m.price.transfer {
            measurement = measurement.add_field("transfer_price", price);
        }

        if let (Some(e), Some(t)) = (m.price.energy, m.price.transfer) {
            measurement = measurement.add_field("price", e + t);
        }

        if !m.temperature.is_nan() {
            measurement = measurement.add_field("temperature", m.temperature);
        }

        if m.resolution == Resolution::Hour {
            measurement = match m.tariff {
                Tariff::Day => measurement.add_tag("tariff", "day"),
                Tariff::Night => measurement.add_tag("tariff", "night"),
            };
        }

        measurement = match m.resolution {
            Resolution::Hour => measurement.add_tag("resolution", "hour"),
            Resolution::Day => measurement.add_tag("resolution", "day"),
            Resolution::Month => measurement.add_tag("resolution", "month"),
            Resolution::Year => measurement.add_tag("resolution", "year"),
        };

        rt.block_on(client.query(&measurement)).expect("influxdb write");
    }
}
