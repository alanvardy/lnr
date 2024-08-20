use reqwest::blocking::Client;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use serde_json::{json, Value};
use spinners::Spinner;
use spinners::Spinners;
use std::collections::HashMap;
use std::env;

use crate::config::Config;

const LINEAR_URL: &str = "https://api.linear.app/graphql";
const CARGO_URL: &str = "https://crates.io/api";
const VERSIONS_URL: &str = "/v1/crates/lnr/versions";

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

pub struct Gql {
    config: Config,
    token: String,
    query: String,
    variables: HashMap<String, Value>,
}

impl Gql {
    /// Creates a new `Gql` instance for executing GraphQL queries.
    ///
    /// # Arguments
    ///
    /// * `config` - A reference to the `Config` containing the necessary configurations.
    /// * `token` - A string slice representing the authentication token.
    /// * `query` - A string slice representing the GraphQL query to be executed.
    ///
    /// # Returns
    ///
    /// Returns a `Gql` instance initialized with the provided parameters.
    pub fn new(config: &Config, token: &str, query: &str) -> Gql {
        Gql {
            config: config.clone(),
            token: token.to_string(),
            query: query.to_string(),
            variables: HashMap::new(),
        }
    }

    /// Inserts a set of variables into the `Gql` instance for use in the GraphQL query.
    ///
    /// # Arguments
    ///
    /// * `variables` - A `HashMap<String, Value>` containing the variables to be used in the GraphQL query.
    ///
    /// # Returns
    ///
    /// Returns the updated `Gql` instance with the provided variables.
    pub fn put_variables(mut self, variables: HashMap<String, Value>) -> Gql {
        self.variables = variables;
        self
    }

    /// Inserts a string variable into the `Gql` instance for use in the GraphQL query.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice representing the key for the variable.
    /// * `value` - A `String` representing the value of the variable.
    ///
    /// # Returns
    ///
    /// Returns the updated `Gql` instance with the provided variable.
    pub fn put_string(mut self, key: &str, value: String) -> Gql {
        self.variables.insert(key.to_string(), Value::String(value));
        self
    }

    /// Inserts an integer variable into the `Gql` instance for use in the GraphQL query.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice representing the key for the variable.
    /// * `value` - A `u8` representing the value of the variable.
    ///
    /// # Returns
    ///
    /// Returns the updated `Gql` instance with the provided variable.
    pub fn put_integer(mut self, key: &str, value: u8) -> Gql {
        self.variables.insert(key.to_string(), json!(value));
        self
    }

    /// Conditionally inserts a string variable into the `Gql` instance if the value is `Some`.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice representing the key for the variable.
    /// * `value` - An `Option<String>` representing the value of the variable. If `None`, the variable is not inserted.
    ///
    /// # Returns
    ///
    /// Returns the updated `Gql` instance with the provided variable if it exists.
    pub fn maybe_put_string(mut self, key: &str, value: Option<String>) -> Gql {
        if let Some(value) = value {
            self.variables.insert(key.to_string(), Value::String(value));
        }
        self
    }

    /// Executes the GraphQL query with the current configuration and variables.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the response as a `String` if successful,
    /// or an error message as a `String`.
    pub fn run(self) -> Result<String, String> {
        let url = get_base_url(&self.config);

        let body = json!({"query": self.query, "variables": self.variables});

        let spinner = maybe_start_spinner(&self.config);
        let response = Client::new()
            .post(url.clone())
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, self.token)
            .json(&body)
            .send()
            .or(Err("Did not get response from server"))?;

        maybe_stop_spinner(spinner);

        if response.status().is_success() {
            Ok(response.text().or(Err("Could not read response text"))?)
        } else {
            Err(format!(
                "
                url: {url}
                ========
                body: {body}
                ========
                Error: {:?}",
                response
            ))
        }
    }
}

/// Retrieves the latest version number of the `lnr` crate from Cargo.io.
///
/// # Returns
///
/// Returns a `Result` containing the latest version number as a `String` if successful,
/// or an error message as a `String`.
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

fn get_base_url(config: &Config) -> String {
    LINEAR_URL.to_string()
}
