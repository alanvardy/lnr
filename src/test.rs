#[cfg(test)]
pub mod fixtures {
    use std::collections::HashMap;

    use crate::config::{self, Config};

    pub fn config() -> Config {
        Config {
            organizations: HashMap::new(),
            path: config::generate_path().unwrap(),
            mock_url: None,
            mock_string: None,
            mock_select: None,
            spinners: Some(true),
        }
    }
}
