use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};

use crate::color;

/// App configuration, serialized as json in $XDG_CONFIG_HOME/lnr.cfg
#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct Config {
    /// List of organizations and their tokens
    pub organizations: HashMap<String, String>,
    /// Path to config file
    pub path: String,
    pub mock_url: Option<String>,
    pub mock_string: Option<String>,
    pub mock_select: Option<usize>,
    // Whether spinners are enabled
    pub spinners: Option<bool>,
}

impl Config {
    pub fn add_organization(&mut self, name: String, token: String) {
        let projects = &mut self.organizations;
        projects.insert(name, token);
    }

    pub fn create(self) -> Result<Config, String> {
        let json = json!(self).to_string();
        let mut file = fs::File::create(&self.path).or(Err("Could not create file"))?;
        file.write_all(json.as_bytes())
            .or(Err("Could not write to file"))?;
        println!("Config successfully created in {}", &self.path);
        Ok(self)
    }

    pub fn load(path: &str) -> Result<Config, String> {
        let mut json = String::new();

        fs::File::open(path)
            .or(Err("Could not find file"))?
            .read_to_string(&mut json)
            .or(Err("Could not read to string"))?;

        serde_json::from_str::<Config>(&json).map_err(|_| format!("Could not parse JSON:\n{json}"))
    }

    pub fn new() -> Result<Config, String> {
        let organizations: HashMap<String, String> = HashMap::new();
        Ok(Config {
            path: generate_path()?,
            spinners: Some(true),
            mock_url: None,
            mock_string: None,
            mock_select: None,
            organizations,
        })
    }

    pub fn remove_organization(&mut self, name: &String) {
        self.organizations.remove(name);
    }

    pub fn organization_names(&self) -> Vec<String> {
        self.organizations.clone().into_keys().collect()
    }

    pub fn token(&self, organization_name: &String) -> Result<String, String> {
        let maybe_org = self
            .organizations
            .clone()
            .into_iter()
            .find(|(k, _v)| k == organization_name);

        match maybe_org {
            Some((_, token)) => Ok(token),
            None => Err("Organization not found".to_string()),
        }
    }

    pub fn save(&mut self) -> std::result::Result<String, String> {
        let json = json!(self);
        let string = serde_json::to_string_pretty(&json).or(Err("Could not convert to JSON"))?;

        fs::OpenOptions::new()
            .write(true)
            .read(true)
            .truncate(true)
            .open(&self.path)
            .or(Err("Could not find config"))?
            .write_all(string.as_bytes())
            .or(Err("Could not write to file"))?;

        Ok(color::green_string("✓"))
    }
}

pub fn get_or_create(config_path: Option<String>) -> Result<Config, String> {
    let path: String = match config_path {
        None => generate_path()?,
        Some(path) => path.trim().to_owned(),
    };

    match fs::File::open(&path) {
        Ok(_) => Config::load(&path),
        Err(_) => Config::new()?.create(),
    }
}

pub fn generate_path() -> Result<String, String> {
    let config_directory = dirs::config_dir()
        .ok_or_else(|| String::from("Could not find config directory"))?
        .to_str()
        .ok_or_else(|| String::from("Could not convert config directory to string"))?
        .to_owned();
    if cfg!(test) {
        _ = fs::create_dir(format!("{config_directory}/lnr_test"));
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 30);
        Ok(format!("tests/{random_string}.testcfg"))
    } else {
        Ok(format!("{config_directory}/lnr.cfg"))
    }
}

#[cfg(test)]
mod tests {

    impl Config {
        /// add the url of the mockito server
        pub fn mock_url(self, url: String) -> Config {
            Config {
                mock_url: Some(url),
                ..self
            }
        }

        // /// Mock out the string response
        // pub fn mock_string(self, string: &str) -> Config {
        //     Config {
        //         mock_string: Some(string.to_string()),
        //         ..self
        //     }
        // }

        // /// Mock out the select response, setting the index of the response
        // pub fn mock_select(self, index: usize) -> Config {
        //     Config {
        //         mock_select: Some(index),
        //         ..self
        //     }
        // }
    }

    use matches::assert_matches;

    use crate::test;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_should_generate_config() {
        let config = Config::new().unwrap();
        assert_eq!(config.organizations, HashMap::new());
    }

    #[test]
    fn add_organization_should_work() {
        let mut config = test::fixtures::config();
        let mut organizations: HashMap<String, String> = HashMap::new();
        assert_eq!(
            config,
            Config {
                path: config.path.clone(),
                organizations: organizations.clone(),
                spinners: Some(true),
                mock_url: None,
                mock_string: None,
                mock_select: None,
            }
        );
        config.add_organization(String::from("test"), "sometoken".to_string());
        organizations.insert(String::from("test"), "sometoken".to_string());
        assert_eq!(
            config,
            Config {
                path: config.path.clone(),
                spinners: Some(true),
                organizations,
                mock_url: None,
                mock_string: None,
                mock_select: None,
            }
        );
    }

    #[test]
    fn remove_project_should_work() {
        let mut organizations: HashMap<String, String> = HashMap::new();
        organizations.insert(String::from("test"), "token1".to_string());
        organizations.insert(String::from("test2"), "token2".to_string());
        let mut config_with_two_projects = Config {
            path: generate_path().unwrap(),
            spinners: Some(true),
            organizations: organizations.clone(),
            mock_url: None,
            mock_string: None,
            mock_select: None,
        };

        assert_eq!(
            config_with_two_projects,
            Config {
                path: config_with_two_projects.path.clone(),
                spinners: Some(true),
                organizations: organizations.clone(),
                mock_url: None,
                mock_string: None,
                mock_select: None,
            }
        );
        config_with_two_projects.remove_organization(&String::from("test"));
        let mut organizations: HashMap<String, String> = HashMap::new();
        organizations.insert(String::from("test2"), "token2".to_string());
        assert_eq!(
            config_with_two_projects,
            Config {
                path: config_with_two_projects.path.clone(),
                organizations,
                spinners: Some(true),
                mock_url: None,
                mock_string: None,
                mock_select: None,
            }
        );
    }

    #[test]
    fn config_tests() {
        // These need to be run sequentially as they write to the filesystem.

        let server = mockito::Server::new();

        // create and load
        let new_config = test::fixtures::config();
        let created_config = new_config.clone().create().unwrap();
        assert_eq!(new_config, created_config);
        let loaded_config = Config::load(&new_config.path).unwrap();
        assert_eq!(created_config, loaded_config);

        // get_or_create (create)
        let config = get_or_create(None);
        assert_eq!(
            config,
            Ok(Config {
                organizations: HashMap::new(),
                path: config.clone().unwrap().path,
                spinners: Some(true),
                mock_url: None,
                mock_string: None,
                mock_select: None,
            })
        );
        delete_config(&config.unwrap().path);

        // get_or_create (load)
        test::fixtures::config()
            .mock_url(server.url())
            .create()
            .unwrap();

        let config = get_or_create(None);

        assert_eq!(
            config,
            Ok(Config {
                organizations: HashMap::new(),
                path: config.clone().unwrap().path,
                spinners: Some(true),
                mock_url: None,
                mock_string: None,
                mock_select: None,
            })
        );
        delete_config(&config.unwrap().path);
    }

    fn delete_config(path: &str) {
        assert_matches!(fs::remove_file(path), Ok(_));
    }
}
