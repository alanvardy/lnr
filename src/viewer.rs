use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::input;
use crate::request;

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
struct ViewerData {
    data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data {
    viewer: Viewer,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Viewer {
    pub id: String,
    name: String,
    team_memberships: TeamMemberships,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TeamMemberships {
    nodes: Vec<TeamNode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TeamNode {
    team: Team,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Team {
    name: String,
    pub id: String,
    projects: ProjectNode,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProjectNode {
    nodes: Vec<Project>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    name: String,
    pub id: String,
}

pub fn get_viewer(config: &Config, token: &String) -> Result<Viewer, String> {
    let json = request::gql(config, token, FETCH_IDS_DOC, HashMap::new())?;

    let result: Result<ViewerData, _> = serde_json::from_str(&json);
    match result {
        Ok(body) => Ok(body.data.viewer),
        Err(err) => Err(format!("Could not parse response for item: {err:?}")),
    }
}

/// Fetch all the team names for a viewer
pub fn team_names(viewer: &Viewer) -> Result<Vec<String>, String> {
    let nodes = viewer.team_memberships.nodes.clone();
    if nodes.is_empty() {
        return Err(String::from("No teams found"));
    };

    let names = nodes.into_iter().map(|n| n.team.name).collect();

    Ok(names)
}

/// Fetch the project names for a team
pub fn project_names(team: &Team) -> Result<Vec<String>, String> {
    let project_names = team
        .projects
        .nodes
        .clone()
        .into_iter()
        .map(|n| n.name)
        .collect();

    Ok(project_names)
}
/// Fetch the team by name
pub fn team_by_name(viewer: &Viewer, team_name: &String) -> Result<Team, String> {
    let nodes = viewer.team_memberships.nodes.clone();
    if nodes.is_empty() {
        return Err(String::from("No teams found"));
    };

    match nodes.into_iter().find(|n| &n.team.name == team_name) {
        None => Err(String::from("Team not found")),
        Some(team_node) => Ok(team_node.team),
    }
}

pub fn team(viewer: &Viewer) -> Result<Team, String> {
    let mut team_names = team_names(viewer)?;

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

pub fn project(team: &Team, project_name: String) -> Result<Option<Project>, String> {
    if project_name.as_str() == "None" {
        return Ok(None);
    }

    match team
        .projects
        .nodes
        .clone()
        .into_iter()
        .find(|n| n.name == project_name)
    {
        Some(project) => Ok(Some(project)),
        None => Err(String::from("Project not found")),
    }
}
