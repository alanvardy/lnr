use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use serde_json::json;
use spinners::Spinner;
use spinners::Spinners;
use std::collections::HashMap;
use std::env;

use crate::config::Config;

const LINEAR_URL: &str = "https://api.linear.app/graphql";
const CARGO_URL: &str = "https://crates.io/api";
const VERSIONS_URL: &str = "/v1/crates/linear_templater/versions";

const SPINNER: Spinners = Spinners::Dots4;
const MESSAGE: &str = "Querying API";

#[derive(Deserialize)]
struct CargoResponse {
    versions: Vec<Version>,
}

#[derive(Deserialize)]
struct Version {
    num: String,
}

pub fn gql(
    config: &Config,
    token: &String,
    query: &str,
    variables: HashMap<String, String>,
) -> Result<String, String> {
    let authorization: &str = &format!("Bearer {token}");

    let body = json!({"query": query, "variables": variables});

    let spinner = maybe_start_spinner(config);
    let response = Client::new()
        .post(LINEAR_URL)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, authorization)
        .json(&body)
        .send()
        .or(Err("Did not get response from server"))?;

    maybe_stop_spinner(spinner);

    if response.status().is_success() {
        Ok(response.text().or(Err("Could not read response text"))?)
    } else {
        Err(format!("Error: {:#?}", response.text()))
    }
}

/// Get latest version number from Cargo.io
pub fn get_latest_version() -> Result<String, String> {
    let request_url = format!("{CARGO_URL}{VERSIONS_URL}");

    let response = Client::new()
        .get(request_url)
        .header(USER_AGENT, "GPTO")
        .send()
        .or(Err("Did not get response from server"))?;

    if response.status().is_success() {
        let cr: CargoResponse =
            serde_json::from_str(&response.text().or(Err("Could not read response text"))?)
                .or(Err("Could not serialize to CargoResponse"))?;
        Ok(cr.versions.first().unwrap().num.clone())
    } else {
        Err(format!("Error: {:#?}", response.text()))
    }
}

fn maybe_start_spinner(config: &Config) -> Option<Spinner> {
    match env::var("DISABLE_SPINNER") {
        Ok(_) => None,
        _ => {
            if let Some(true) = config.spinners {
                let sp = Spinner::new(SPINNER, MESSAGE.into());
                Some(sp)
            } else {
                None
            }
        }
    }
}
fn maybe_stop_spinner(spinner: Option<Spinner>) {
    if let Some(mut sp) = spinner {
        sp.stop();
        print!("\x1b[2K\r");
    };
}
