use std::io;

use influent::client::{Client, Credentials, Precision};
use influent::measurement::{Measurement, Value};

use energiatili_model::measurement::{Measurements, Resolution, Tariff};
use energiatili_model::model::Model;

fn main() {
    env_logger::init();

    let stdin = io::stdin();
    let model = Model::from_reader(stdin).expect("read JSON Model");
    let measurements = Measurements::from(&model);

    let credentials = Credentials {
        username: "",
        password: "",
        database: "energiatili",
    };

    let influxdb = influent::create_client(credentials, vec!["http://127.0.0.1:8086"]);

    const SIZE: usize = 1000;
    let mut buf = Vec::with_capacity(SIZE);

    for m in measurements.0 {
        let mut measurement = Measurement::new("energiatili");

        let ts = m.timestamp.timestamp_nanos();
        measurement.set_timestamp(ts);

        measurement.add_field("consumption", Value::Float(m.consumption));
        measurement.add_field("quality", Value::Integer(i64::from(m.quality)));

        if let Some(price) = m.price.energy {
            measurement.add_field("energy_price", Value::Float(price));
        }

        if let Some(price) = m.price.transfer {
            measurement.add_field("transfer_price", Value::Float(price));
        }

        if let (Some(e), Some(t)) = (m.price.energy, m.price.transfer) {
            measurement.add_field("price", Value::Float(e + t));
        }

        if !m.temperature.is_nan() {
            measurement.add_field("temperature", Value::Float(m.temperature));
        }

        if m.resolution == Resolution::Hour {
            match m.tariff {
                Tariff::Day => measurement.add_tag("tariff", "day"),
                Tariff::Night => measurement.add_tag("tariff", "night"),
            }
        }

        match m.resolution {
            Resolution::Hour => measurement.add_tag("resolution", "hour"),
            Resolution::Day => measurement.add_tag("resolution", "day"),
            Resolution::Month => measurement.add_tag("resolution", "month"),
            Resolution::Year => measurement.add_tag("resolution", "year"),
        }

        buf.push(measurement);

        if buf.len() == SIZE {
            influxdb.write_many(&buf, None).expect("influent write_many");
            buf.clear();
        }
    }

    influxdb.write_many(&buf, None).expect("influent write_many");
}
