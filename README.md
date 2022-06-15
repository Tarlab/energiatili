# Tools to export and process data from Energiatili

## How to compile and install

 1. Install Rust & Cargo for compilation. Easiest way is probably to use
    https://rustup.rs/

     Or you can use Docker (https://hub.docker.com/_/rust) if your target system
     is Linux.

 2. Compile & install: `cargo install --path energiatili-import && cargo install --path influxdb-export`

 3. You should find the binaries installed in `${HOME}/.cargo/bin`. Move them
    wheverever you need.

## Configuration file

Under the directory specified (see
https://docs.rs/dirs/2.0.2/dirs/fn.config_dir.html, or just run one of the
binaries and the error message should lead the way), add file called
`energiatili.toml` with following kind of config:

```toml
[energiatili]
username = "<your username>"
password = "<your really secret password>"

[influxdb]
url = "<url to InfluxDB, e.g. http://127.0.0.1:8086>"
token = "<access token to InfluxDb>"
org = "<organization in InfluxDB>"
bucket = "<data bucket in InfluxDB>"
```

# Tools

## energiatili-import

This tool logs in to [Energiatili](https://www.energiatili.fi/) and downloads the electricity
usage information. This information is parsed and then outputted as JSON to
stdout.

## influxdb-export

A tool that takes the output of `energiatili-import` and pushes the data into
[InfluxDB](https://en.wikipedia.org/wiki/InfluxDB) database.

# Libraries

## energiatili-model

A library which contains `Model` which is the parsed data structure from Energiatili and `Measurements` which is data structure used to feeding InfluxDB.

## energiatili-config

A library which contains functionality to read the configuration file.
