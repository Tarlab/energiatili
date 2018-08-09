extern crate cookie;
extern crate env_logger;
extern crate reqwest;
extern crate serde_json;

#[macro_use]
extern crate log;

extern crate energiatili_model;

use std::io;

use reqwest::header;
use cookie::CookieJar;

use energiatili_model::model::Model;

const BASE_URL: &str = "https://www.energiatili.fi";
const FIRST_URL: &str = "/Extranet/Extranet";
const LOGIN_URL: &str = "/Extranet/Extranet/LogIn";
const REPORT_URL: &str = "/Reporting/CustomerConsumption/UserConsumptionReport";

mod secrets;

fn main() -> io::Result<()> {
    env_logger::init();

    let mut client = Client::new();
    client.login().expect("Client login");
    let report = client.consumption_report();
    debug!("Consumption report HTML:\n{}\n", report);

    let cursor = io::Cursor::new(report);
    let model = Model::from_report_html(cursor);
    println!("{}", serde_json::to_string_pretty(&model)?);
    Ok(())
}

#[derive(Debug)]
struct Client {
    jar: CookieJar,
    client: reqwest::Client,
}

impl Client {
    fn new() -> Client {
        trace!("Client::new()");
        let client = reqwest::ClientBuilder::new()
            .build().expect("ClientBuilder::build");
        let mut jar = cookie::CookieJar::new();

        let req = client.get(&format!("{}{}", BASE_URL, FIRST_URL))
            .build().expect("Build first request");
        debug!("GET Request: {:?}", req);

        let resp = client.execute(req).expect("Execute consumption report request");
        debug!("GET Response: {:?}", resp);

        if let Some(cookies) = resp.headers().get::<header::SetCookie>() {
            store_cookies(&mut jar, cookies);
        }

        Self { jar, client }
    }

    fn login(&mut self) -> io::Result<()> {
        trace!("Client::login({:?})", self);
        let jar = &mut self.jar;

        let client = reqwest::ClientBuilder::new()
            .redirect(reqwest::RedirectPolicy::none())
            .build().expect("ClientBuilder::build");

        let params = [("username", secrets::USERNAME), ("password", secrets::PASSWORD)];
        let headers = cookie_header(&jar);
        let req = client.post(&format!("{}{}", BASE_URL, LOGIN_URL))
            .form(&params)
            .headers(headers)
            .build().expect("build login request");
        debug!("POST Request: {:?}", req);

        let resp = client.execute(req).expect("Execute consumption report request");
        debug!("POST Response: {:?}", resp);

        if let Some(cookies) = resp.headers().get::<header::SetCookie>() {
            store_cookies(jar, cookies);
        }

        Ok(())
    }

    fn consumption_report(&self) -> String {
        trace!("Client::consumption_report({:?})", self);
        let client = &self.client;
        let jar = &self.jar;

        let headers = cookie_header(&jar);
        let req = client.get(&format!("{}{}", BASE_URL, REPORT_URL))
            .headers(headers)
            .build().expect("Build consumption report request");
        debug!("GET Request: {:?}", req);

        let mut resp = client.execute(req).expect("Execute consumption report request");
        debug!("GET Response: {:?}", resp);

        let mut buf: Vec<u8> = Vec::new();
        resp.copy_to(&mut buf).expect("Result copy_to");
        let res = String::from_utf8_lossy(&buf);
        res.to_string()
    }
}

fn cookie_header(jar: &CookieJar) -> header::Headers {
    trace!("cookie_header(...)");
    let mut headers = header::Headers::new();
    let mut c = reqwest::header::Cookie::new();

    for cookie in jar.iter() {
        c.append(cookie.name().to_string(), cookie.value().to_string());
        debug!("Adding cookie into request: {}", cookie);
    }

    headers.set::<header::Cookie>(c);
    headers
}

fn store_cookies(jar: &mut CookieJar, cookies: &header::SetCookie) {
    trace!("store_cookies(...)");
    for cookie in cookies.iter() {
        let cookie = cookie.clone();
        let c = cookie::Cookie::parse(cookie).expect("parse cookie");
        debug!("Adding cookie into jar: {} = {}", c.name(), c.value());
        jar.add(c);
    }
}
