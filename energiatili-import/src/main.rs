use std::io;

use log::{debug, trace};

use energiatili_model::model::Model;

const BASE_URL: &str = "https://www.energiatili.fi";
const FIRST_URL: &str = "/Extranet/Extranet";
const LOGIN_URL: &str = "/Extranet/Extranet/LogIn";
const REPORT_URL: &str = "/Reporting/CustomerConsumption/UserConsumptionReport";

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
    let username = &config.energiatili.username;
    let password = &config.energiatili.password;

    match run(username, password).await {
        Ok(()) => (),
        Err(err) => {
            eprintln!("ERROR: {}", err);
            std::process::exit(1);
        }
    }
}

async fn run(username: &str, password: &str) -> Result<()> {
    let mut client = Client::new().await?;
    client.login(username, password).await?;
    let report = client.consumption_report().await?;
    debug!("Consumption report HTML:\n{}\n", report);

    let cursor = io::Cursor::new(report);
    let model = Model::from_report_html(cursor);
    println!("{}", serde_json::to_string_pretty(&model)?);
    Ok(())
}

#[derive(Debug)]
struct Client {
    client: reqwest::Client,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum Error {
    JSON(serde_json::Error),
    Reqwest(reqwest::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::JSON(err) => err.source(),
            Error::Reqwest(err) => err.source(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::JSON(err) => writeln!(f, "JSON error: {}", err),
            Error::Reqwest(err) => writeln!(f, "HTTP error: {}", err),
        }
    }
}

impl Client {
    async fn new() -> Result<Client> {
        trace!("Client::new()");
        let client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .cookie_store(true)
            .build()?;

        let url = format!("{}{}", BASE_URL, FIRST_URL);
        let req = client.get(&url).build()?;
        debug!("GET Request: {:?}", req);

        let resp = client.execute(req).await?;
        debug!("GET Response: {:?}", resp);

        Ok(Self { client })
    }

    async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        trace!("Client::login({:?})", self);

        let url = format!("{}{}", BASE_URL, LOGIN_URL);
        let params = [("username", username), ("password", password)];
        let req = self.client.post(&url).form(&params).build()?;
        debug!("POST Request: {:?}", req);

        let resp = self.client.execute(req).await?;
        debug!("POST Response: {:?}", resp);

        Ok(())
    }

    async fn consumption_report(&self) -> Result<String> {
        trace!("Client::consumption_report({:?})", self);
        let client = &self.client;

        let url = format!("{}{}", BASE_URL, REPORT_URL);
        let req = client.get(&url).build()?;
        debug!("GET Request: {:?}", req);

        let resp = client.execute(req).await?;
        debug!("GET Response: {:?}", resp);

        let res = resp.text().await?;
        Ok(res)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JSON(err)
    }
}
