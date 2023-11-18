#[cfg(test)]
pub mod fixtures {
    use std::collections::HashMap;

    use crate::{
        config::{self, Config},
        team::{ProjectNode, State, StateNode, Team},
    };

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
    pub fn team() -> Team {
        Team {
            name: "Thundercats".to_string(),
            id: "123456".to_string(),
            projects: Some(ProjectNode { nodes: Vec::new() }),
            states: Some(StateNode { nodes: Vec::new() }),
        }
    }
    pub fn state() -> State {
        State {
            name: "Thundercats".to_string(),
            id: "123456".to_string(),
            position: 1,
        }
    }
}
#[cfg(test)]
pub mod responses {
    pub fn issue_create() -> String {
        "{
            \"data\":{
              \"issueCreate\":{
                \"issue\":{
                  \"id\":\"cbe16d8a-9999-9999-9999-9f2e79c3cb7e\",
                  \"identifier\":\"BE-3354\",
                  \"title\":\"Test\",
                  \"description\":null,
                  \"url\":\"https://linear.app/vardy/issue/BE-3354/test\",
                  \"branchName\":\"be-3354-test\",
                  \"state\":{
                    \"id\":\"eb19df39-9999-9999-9999-b698543f01ce\",
                    \"position\":1,
                    \"name\":\"Triage\"
                  }
                }
              }
            }
          }\n"
        .to_string()
    }

    pub fn issue_list() -> String {
        "{\"data\":
            {\"issues\":{
              \"nodes\":[
                {
                    \"id\":\"438bced3-9999-9999-9999-a51423f24fc6\",
                    \"identifier\":\"SHO-2148\",
                    \"title\":\"Modify schema\",
                    \"description\":\"* Make item_name_id nullable\\n* Add non-null field for listing_url\\n* Unique index on listing_url\\n\\n* [ ] Migration\\n* [ ] Change schema\\n* [ ] Add to GQL type\",
                    \"url\":\"https://linear.app/vardy/issue/SHO-2148/modify-schema\",
                    \"branchName\":\"sho-2148-modify-schema\",
                    \"children\":{
                      \"nodes\":[]
                    },
                    \"state\":{
                      \"id\":\"7a890819-9999-9999-9999-b1abe79c2b8e\",
                        \"position\":1,
                      \"name\":\"Todo\"
                    }
                  }
                ]
              }
            }
        }\n".to_string()
    }
}
