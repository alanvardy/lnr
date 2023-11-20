use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
extern crate walkdir;

use walkdir::WalkDir;

use crate::config::Config;
use crate::priority::{self, Priority};
use crate::request;
use crate::team::{Project, State, Team};
use crate::viewer::Viewer;

const ISSUE_CREATE_DOC: &str = "mutation (
                    $title: String!
                    $teamId: String!
                    $priority: Int
                    $assigneeId: String
                    $description: String,
                    $parentId: String
                    $stateId: String
                    $projectId: String
                ) {
                issueCreate(
                    input: {
                        title: $title
                        priority: $priority
                        teamId: $teamId
                        assigneeId: $assigneeId
                        stateId: $stateId
                        description: $description
                        parentId: $parentId
                        projectId: $projectId
                    }
                ) {
                    issue {
                        id
                        url
                    }
                }
                }
                ";

#[derive(Deserialize)]
struct Template {
    parent: ParentIssue,
    children: Option<Vec<ChildIssue>>,
    variables: HashMap<String, String>,
}

#[derive(Deserialize)]
struct ParentIssue {
    title: String,
    description: Option<String>,
}
#[derive(Deserialize)]
struct ChildIssue {
    title: String,
    description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct IssueCreateResponse {
    data: Option<Data>,
}
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct Data {
    issueCreate: IssueCreate,
}

#[derive(Deserialize, Serialize, Debug)]
struct IssueCreate {
    issue: Issue,
}

#[derive(Deserialize, Serialize, Debug)]
struct Issue {
    id: String,
    url: String,
}

/// We want to support a file path or a directory
#[allow(clippy::too_many_arguments)]
pub fn evaluate(
    config: &Config,
    token: &str,
    team: &Team,
    project: &Option<Project>,
    viewer: &Viewer,
    path: &String,
    state: &State,
    priority: &Priority,
) -> Result<String, String> {
    if Path::is_dir(Path::new(&path)) {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(is_issue_toml)
        {
            create_issues(
                config,
                token,
                team,
                viewer,
                project,
                &entry.path().to_str().unwrap().to_string(),
                state,
                priority,
            )?;
        }
        Ok("Done".to_string())
    } else {
        create_issues(config, token, team, viewer, project, path, state, priority)
    }
}

#[allow(clippy::too_many_arguments)]
fn create_issues(
    config: &Config,
    token: &str,
    team: &Team,
    viewer: &Viewer,
    project: &Option<Project>,
    path: &String,
    state: &State,
    priority: &Priority,
) -> Result<String, String> {
    let mut toml_string = String::new();

    fs::File::open(path.clone())
        .or(Err("Could not find file"))?
        .read_to_string(&mut toml_string)
        .or(Err("Could not read to string"))?;

    println!("Processing {path}");

    let Template {
        parent,
        children,
        variables,
    } = toml::from_str(&toml_string).unwrap();

    let title = fill_in_variables(parent.title.clone(), variables.clone())?;
    let description_template = parent.description.unwrap_or_default();
    let description = fill_in_variables(description_template, variables.clone())?;
    let project_id = project.clone().map(|p| p.id);
    let priority = priority::priority_to_int(priority);

    let response = request::Gql::new(config, token, ISSUE_CREATE_DOC)
        .put_string("title", title)
        .put_string("teamId", team.id.clone())
        .put_integer("priority", priority)
        .put_string("stateId", state.id.clone())
        .put_string("assigneeId", viewer.id.clone())
        .put_string("description", description)
        .maybe_put_string("projectId", project_id.clone())
        .run()?;

    let Issue { id, url } = extract_id_from_response(response)?;

    println!("- [{}] {}", id, url);

    for child in children.unwrap_or_default().iter() {
        let title = fill_in_variables(child.title.clone(), variables.clone())?;
        let child_description_template = child.description.clone().unwrap_or_default();
        let child_description = fill_in_variables(child_description_template, variables.clone())?;

        let response = request::Gql::new(config, token, ISSUE_CREATE_DOC)
            .put_string("title", title)
            .put_string("teamId", team.id.clone())
            .put_string("stateId", state.id.clone())
            .put_string("parentId", id.clone())
            .put_integer("priority", priority)
            .put_string("assigneeId", viewer.id.clone())
            .put_string("description", child_description)
            .maybe_put_string("projectId", project_id.clone())
            .run()?;
        let Issue { id, url } = extract_id_from_response(response)?;
        println!("  - [{}] {}", id, url);
    }
    Ok("Done".to_string())
}

/// Returns true if it is a TOML file that can be processed
fn is_issue_toml(entry: &walkdir::DirEntry) -> bool {
    entry.file_name().to_str().unwrap().ends_with(".toml")
        && !entry.file_name().to_str().unwrap().contains("Cargo.toml")
}

/// Get the id from an issue response, needed for parent issues and terminal output
fn extract_id_from_response(response: String) -> Result<Issue, String> {
    let data: Result<IssueCreateResponse, _> = serde_json::from_str(&response);

    match data {
        Ok(IssueCreateResponse {
            data: Some(Data {
                issueCreate: IssueCreate { issue },
            }),
        }) => Ok(issue),
        err => Err(format!(
            "Could not parse response for issue:
            ---
            {err:?}
            ---
            {response:?}"
        )),
    }
}

fn fill_in_variables(
    template: String,
    variables: HashMap<String, String>,
) -> Result<String, String> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);

    handlebars
        .register_template_string("t1", template)
        .map_err(|e| format!("Could not register template: {e:?}"))?;

    handlebars
        .render("t1", &json!(variables))
        .map_err(|e| format!("Could not render template: {e:?}"))
}
