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

/// Represents the response data structure for querying a team's states.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TeamData {
    data: Data,
}

/// Represents the data structure containing team information in the response.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data {
    team: Team,
}

/// Describes a Linear team and its various projects.
#[derive(Serialize, Default, Deserialize, Debug, Clone)]
pub struct Team {
    /// The name of the team.
    pub name: String,
    /// The unique identifier of the team.
    pub id: String,
    /// An optional collection of projects associated with the team.
    pub projects: Option<ProjectNode>,
    /// An optional collection of states associated with the team.
    pub states: Option<StateNode>,
}

/// Represents a collection of projects within a team.
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct ProjectNode {
    /// A list of projects associated with the team.
    pub nodes: Vec<Project>,
}

/// Represents a single project within a team.
#[derive(Serialize, Default, Deserialize, Debug, Clone)]
pub struct Project {
    /// The name of the project.
    pub name: String,
    /// The unique identifier of the project.
    pub id: String,
}

/// Represents a collection of states within a team.
#[derive(Serialize, Default, Deserialize, Debug, Clone)]
pub struct StateNode {
    /// A list of states associated with the team.
    pub nodes: Vec<State>,
}

/// Represents a single state within a team.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    /// The name of the state.
    pub name: String,
    /// The unique identifier of the state.
    pub id: String,
    /// The position of the state, used for ordering.
    pub position: f32,
}

impl Display for State {
    /// Formats the `State` for display, typically used in selection interfaces.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable reference to the formatter.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating whether the formatting was successful.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name.clone();
        write!(f, "{name}")
    }
}

/// Retrieves the states associated with a given team from the Linear API.
///
/// # Arguments
///
/// * `config` - A reference to the `Config` containing the necessary configurations.
/// * `token` - A string slice representing the authentication token.
/// * `team` - A reference to the `Team` whose states are to be retrieved.
///
/// # Returns
///
/// Returns a `Result` containing a `Vec<State>` if successful, or an error message as a `String`.
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

