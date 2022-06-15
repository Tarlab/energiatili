use std::fs;
use std::io;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub energiatili: Energiatili,
    pub influxdb: InfluxDB,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Energiatili {
    pub username: String,
    pub password: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct InfluxDB {
    pub url: String,
    pub token: String,
    pub org: String,
    pub bucket: String,
}

impl Config {
    pub fn read() -> io::Result<Config> {
        let mut config_dir = dirs::config_dir().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Couldn't figure out where config file should be. Complain loudly.",
            )
        })?;
        config_dir.push("energiatili.config");
        let buf = fs::read(&config_dir).map_err(|err| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Couldn't open file {}: {}.", config_dir.display(), err),
            )
        })?;
        let config = toml::from_slice(&buf).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Couldn't parse file {}: {}.", config_dir.display(), err),
            )
        })?;
        Ok(config)
    }

    pub fn example() -> String {
        let config = Config::default();
        toml::to_string_pretty(&config).unwrap()
    }
}
