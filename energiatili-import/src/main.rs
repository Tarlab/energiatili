use std::io;

use log::{debug, trace};

use energiatili_model::model::Model;

const BASE_URL: &str = "https://www.energiatili.fi";
const FIRST_URL: &str = "/Extranet/Extranet";
const LOGIN_URL: &str = "/Extranet/Extranet/LogIn";
const REPORT_URL: &str = "/Reporting/CustomerConsumption/UserConsumptionReport";

fn main() -> io::Result<()> {
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

    let mut client = Client::new().expect("new client");
    client.login(username, password).expect("Client login");
    let report = client.consumption_report().expect("consumption_report");
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

#[derive(Debug)]
enum Error {
    Reqwest(reqwest::Error),
}

type Result<T> = std::result::Result<T, Error>;

impl Client {
    fn new() -> Result<Client> {
        trace!("Client::new()");
        let client = reqwest::ClientBuilder::new()
            .redirect(reqwest::RedirectPolicy::none())
            .cookie_store(true)
            .build()?;

        let req = client.get(&format!("{}{}", BASE_URL, FIRST_URL))
            .build()?;
        debug!("GET Request: {:?}", req);

        let resp = client.execute(req)?;
        debug!("GET Response: {:?}", resp);

        Ok(Self { client })
    }

    fn login(&mut self, username: &str, password: &str) -> Result<()> {
        trace!("Client::login({:?})", self);

        let params = [("username", username), ("password", password)];
        let req = self.client.post(&format!("{}{}", BASE_URL, LOGIN_URL))
            .form(&params)
            .build()?;
        debug!("POST Request: {:?}", req);

        let resp = self.client.execute(req)?;
        debug!("POST Response: {:?}", resp);

        Ok(())
    }

    fn consumption_report(&self) -> Result<String> {
        trace!("Client::consumption_report({:?})", self);
        let client = &self.client;

        let req = client.get(&format!("{}{}", BASE_URL, REPORT_URL))
            .build().expect("Build consumption report request");
        debug!("GET Request: {:?}", req);

        let mut resp = client.execute(req)?;
        debug!("GET Response: {:?}", resp);

        let mut buf: Vec<u8> = Vec::new();
        resp.copy_to(&mut buf)?;
        let res = String::from_utf8(buf).expect("from_utf8");
        Ok(res)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err)
    }
}
