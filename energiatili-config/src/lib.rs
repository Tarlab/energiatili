use std::fs;
use std::io;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub energiatili: Energiatili,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Energiatili {
    pub username: String,
    pub password: String,
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
        let Ok(s) = std::str::from_utf8(&buf) else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Couldn't read file as UTF-8 {}.", config_dir.display()),
            ));

        };
        let config = toml::from_str(s).map_err(|err| {
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
