use std::io::{BufRead, Read};

use chrono::{DateTime, Utc};
use serde_json;

use crate::utils::fix_new_date;

impl Model {
    pub fn from_report_html(input: impl BufRead) -> Model {
        let mut lines = input.lines();
        while let Some(Ok(line)) = lines.next() {
            if let Some(start) = line.find("var model = ") {
                let json_str = fix_new_date(&line[start + 12..line.len() - 1]);
                let model: Model = serde_json::from_str(&json_str).expect("serde_json::from_str");
                return model;
            }
        }
        panic!("Model not found");
    }

    pub fn from_reader(input: impl Read) -> Result<Model, serde_json::Error> {
        serde_json::from_reader(input)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Model {
    pub hours: OneResolution,
    pub days: OneResolution,
    pub months: OneResolution,
    pub years: OneResolution,
    pub network_price_list: PriceList,
    pub sales_price_list: PriceList,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OneResolution {
    pub consumptions: Vec<Consumption>,
    pub consumption_statuses: Measurements,
    pub temperature: Measurements,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Consumption {
    pub series: Measurements,
    pub tariff_time_zone_name: String,
}

/// Measurements are shared between many different collection of values
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Measurements {
    pub data: Vec<serde_json::value::Value>,
    pub data_count: usize,
    pub name: String,
    pub resolution: String,
    pub start: DateTime<Utc>,
    pub stop: DateTime<Utc>,
    #[serde(rename = "Type")]
    pub type_: String,
    pub unit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PriceList {
    pub time_based_energy_day_prices: Vec<Price>,
    pub time_based_energy_night_prices: Vec<Price>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Price {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub price_no_vat: f64,
    pub price_with_vat: f64,
}
