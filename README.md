# Tools to export and process data from Energiatili

# energiatili-import

This tool logs in to [Energiatili](https://www.energiatili.fi/) and downloads the electricity
usage information. This information is parsed and then outputted as JSON to
stdout.

# influxdb-export

A tool that takes the output of `energiatili-import` and pushes the data into
[InfluxDB](https://en.wikipedia.org/wiki/InfluxDB) database.

# energiatili-model

A library which contains `Model` which is the parsed data structure from Energiatili and `Measurements` which is data structure used to feeding InfluxDB.
