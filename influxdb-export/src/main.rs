use std::io;

use energiatili_model::measurement::{Measurements, Resolution, Tariff};
use energiatili_model::model::Model;
use influxdb2_client::models::data_point::{DataPoint, DataPointError};

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = match energiatili_config::Config::read() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("ERROR: {}", err);
            std::process::exit(1);
        }
    };
    let db_url = config.influxdb.url;
    let db_token = config.influxdb.token;
    let db_org = config.influxdb.org;
    let db_bucket = config.influxdb.bucket;

    let stdin = io::stdin().lock();
    let model = Model::from_reader(stdin).expect("read JSON Model");
    let measurements = Measurements::from(&model);
    let dp_stream = DataPointStream::new(measurements);

    let client = influxdb2_client::Client::new(db_url, db_token);
    client
        .write(&db_org, &db_bucket, dp_stream)
        .await
        .expect("write to influxdb");
}

struct DataPointStream {
    measurements: std::collections::btree_set::IntoIter<Measurement>,
}

impl DataPointStream {
    fn new(measurements: Measurements) -> Self {
        Self {
            measurements: measurements.0.into_iter(),
        }
    }
}

use std::pin::Pin;
use std::task::{Context, Poll};

impl futures::Stream for DataPointStream {
    type Item = DataPoint;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.measurements.size_hint()
    }

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let measurement = match self.measurements.next() {
            Some(m) => m,
            None => return Poll::Ready(None),
        };
        let dp = convert(&measurement).expect("convert measurement to datapoint");
        Poll::Ready(Some(dp))
    }
}

use energiatili_model::measurement::Measurement;

fn convert(m: &Measurement) -> Result<DataPoint, DataPointError> {
    let mut dp = influxdb2_client::models::DataPoint::builder("energiatili");

    dp = dp.timestamp(m.timestamp.timestamp_nanos());
    dp = dp.field("consumption", m.consumption);
    dp = dp.field("quality", i64::from(m.quality));

    if let Some(price) = m.price.energy {
        dp = dp.field("energy_price", price);
    }

    if let Some(price) = m.price.transfer {
        dp = dp.field("transfer_price", price);
    }

    if let (Some(e), Some(t)) = (m.price.energy, m.price.transfer) {
        dp = dp.field("price", e + t);
    }

    if !m.temperature.is_nan() {
        dp = dp.field("temperature", m.temperature);
    }

    if m.resolution == Resolution::Hour {
        dp = match m.tariff {
            Tariff::Day => dp.tag("tariff", "day"),
            Tariff::Night => dp.tag("tariff", "night"),
            Tariff::Simple => dp,
        };
    }

    dp = match m.resolution {
        Resolution::Hour => dp.tag("resolution", "hour"),
        Resolution::Day => dp.tag("resolution", "day"),
        Resolution::Month => dp.tag("resolution", "month"),
        Resolution::Year => dp.tag("resolution", "year"),
    };

    dp.build()
}
