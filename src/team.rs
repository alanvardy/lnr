use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{config::Config, request};

const TEAM_STATES_DOC: &str = "
        query ($id: String!) {
            team (id: $id) {
                name
                id
                states {
                    nodes {
                        name
                        id
                        position
                    }
                }
            }
        }";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TeamData {
    data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data {
    team: Team,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone)]
pub struct Team {
    pub name: String,
    pub id: String,
    pub projects: Option<ProjectNode>,
    pub states: Option<StateNode>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ProjectNode {
    pub nodes: Vec<Project>,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone)]
pub struct Project {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Default, Deserialize, Debug, Clone)]
pub struct StateNode {
    pub nodes: Vec<State>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    pub name: String,
    pub id: String,
    pub position: f32,
}

impl Display for State {
    /// This is for rendering in a select
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name.clone();
        write!(f, "{name}")
    }
}
pub fn get_states(config: &Config, token: &str, team: &Team) -> Result<Vec<State>, String> {
    let response = request::Gql::new(config, token, TEAM_STATES_DOC)
        .put_string("id", team.id.clone())
        .run()?;
    let result: Result<TeamData, _> = serde_json::from_str(&response);
    match result {
        Ok(body) => {
            let mut states = body.data.team.states.unwrap().nodes;
            states.sort_unstable_by_key(|s| s.position as i32);
            Ok(states)
        }
        Err(err) => Err(format!("Could not parse response for states: {err:?}")),
    }
}
