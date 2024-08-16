use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::input;
use crate::request;
use crate::{Project, Team};

const FETCH_IDS_DOC: &str = "
        query {
            viewer {
                name
                id
                teamMemberships {
                    nodes {
                        team {
                            name
                            id
                            projects {
                                nodes {
                                    name
                                    id
                                }
                            }
                        }
                    }
                }
            }
        }";

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Represents the top-level structure of the response for fetching viewer data.
struct ViewerData {
    /// The data field containing viewer-related information.
    data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Represents the data structure containing viewer information.
struct Data {
    /// The viewer field containing detailed information about the viewer.
    viewer: Viewer,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// Represents a viewer with their associated teams and projects.
pub struct Viewer {
    /// The unique identifier of the viewer.
    pub id: String,
    /// The name of the viewer.
    name: String,
    /// The teams the viewer is a member of, along with their associated projects.
    team_memberships: TeamMemberships,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Represents the structure containing a list of team memberships.
struct TeamMemberships {
    /// A vector of team nodes, each representing a team the viewer is a member of.
    nodes: Vec<TeamNode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Represents a single team node, containing information about a specific team.
struct TeamNode {
    /// The team associated with this node.
    team: Team,
}

/// Fetches the viewer information, including their teams and projects, from the API.
///
/// # Arguments
///
/// * `config` - A reference to the `Config` containing the necessary configurations.
/// * `token` - A string slice representing the authentication token.
///
/// # Returns
///
/// Returns a `Result` containing the `Viewer` if successful, or an error message as a `String`.
pub fn get_viewer(config: &Config, token: &str) -> Result<Viewer, String> {
    let response = request::Gql::new(config, token, FETCH_IDS_DOC).run()?;

    let result: Result<ViewerData, _> = serde_json::from_str(&response);
    match result {
        Ok(body) => Ok(body.data.viewer),
        Err(err) => Err(format!("Could not parse response for item: {err:?}")),
    }
}

/// Fetches all the team names associated with the viewer.
///
/// # Arguments
///
/// * `viewer` - A reference to the `Viewer` whose teams are to be retrieved.
///
/// # Returns
///
/// Returns a `Result` containing a vector of team names as `Vec<String>` if successful,
/// or an error message as a `String`.
pub fn team_names(viewer: &Viewer) -> Result<Vec<String>, String> {
    let nodes = viewer.team_memberships.nodes.clone();
    if nodes.is_empty() {
        return Err(String::from("No teams found"));
    };

    let names = nodes.into_iter().map(|n| n.team.name).collect();

    Ok(names)
}

/// Fetches the project names for a given team.
///
/// # Arguments
///
/// * `team` - An optional reference to the `Team` whose projects are to be retrieved.
///
/// # Returns
///
/// Returns a `Result` containing a vector of project names as `Vec<String>` if successful,
/// or an empty vector if no projects are found.
pub fn project_names(team: &Option<Team>) -> Result<Vec<String>, String> {
    if let Some(team) = team {
        let project_names = team
            .projects
            .clone()
            .unwrap_or_default()
            .nodes
            .clone()
            .into_iter()
            .map(|n| n.name)
            .collect();

        Ok(project_names)
    } else {
        Ok(Vec::new())
    }
}

/// Fetches the team by its name from the viewer's list of teams.
///
/// # Arguments
///
/// * `viewer` - A reference to the `Viewer` whose teams are to be searched.
/// * `team_name` - A string reference representing the name of the team to be fetched.
///
/// # Returns
///
/// Returns a `Result` containing the `Team` if found, or an error message as a `String`.
pub fn team_by_name(viewer: &Viewer, team_name: &String) -> Result<Team, String> {
    let nodes = viewer.team_memberships.nodes.clone();
    if nodes.is_empty() {
        return Err(String::from("No teams found"));
    };

    match nodes.iter().find(|n| &n.team.name == team_name) {
        None => {
            let team_names = nodes
                .iter()
                .map(|t| t.team.name.clone())
                .collect::<Vec<String>>()
                .join(", ");
            Err(format!(
                "Team {team_name} not found, options are: {team_names}"
            ))
        }
        Some(team_node) => Ok(team_node.team.clone()),
    }
}

/// Fetches the team for the viewer based on an optional team name.
///
/// # Arguments
///
/// * `viewer` - A reference to the `Viewer` whose teams are to be searched.
/// * `team_name` - An optional reference to a string representing the team name to search for.
///
/// # Returns
///
/// Returns a `Result` containing the selected `Team` if found, or an error message as a `String`.
pub fn team(viewer: &Viewer, team_name: &Option<String>) -> Result<Team, String> {
    let mut team_names = team_names(viewer)?;

    if let Some(name) = team_name {
        return team_by_name(viewer, name);
    }

    team_names.sort();

    if team_names.is_empty() {
        Err("No teams found".to_string())
    } else if team_names.len() == 1 {
        team_by_name(viewer, team_names.first().unwrap())
    } else {
        let team_name = input::select("Select a team", team_names, None)?;
        team_by_name(viewer, &team_name)
    }
}

/// Fetches the project by its name from a given team.
///
/// # Arguments
///
/// * `team` - An optional reference to the `Team` whose projects are to be searched.
/// * `project_name` - A `String` representing the name of the project to be fetched.
///
/// # Returns
///
/// Returns a `Result` containing an `Option<Project>` if found, or an error message as a `String`.
pub fn project(team: &Option<Team>, project_name: String) -> Result<Option<Project>, String> {
    if project_name.as_str() == "None" {
        return Ok(None);
    }

    if let Some(team) = team {
        match team
            .projects
            .clone()
            .unwrap_or_default()
            .nodes
            .clone()
            .into_iter()
            .find(|n| n.name == project_name)
        {
            Some(project) => Ok(Some(project)),
            None => Err(String::from("Project not found")),
        }
    } else {
        Ok(None)
    }
}
