use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
extern crate walkdir;

use walkdir::WalkDir;

use crate::request;

const ISSUE_CREATE_DOC: &str = "mutation (
                    $title: String!
                    $teamId: String!
                    $assigneeId: String
                    $description: String,
                    $parentId: String
                    $projectId: String
                ) {
                issueCreate(
                    input: {
                        title: $title
                        teamId: $teamId
                        assigneeId: $assigneeId
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
    children: Vec<ChildIssue>,
    variables: HashMap<String, String>,
}

#[derive(Deserialize)]
struct ParentIssue {
    title: String,
    team_id: String,
    project_id: Option<String>,
    assignee_id: Option<String>,
    description: Option<String>,
}
#[derive(Deserialize)]
struct ChildIssue {
    title: String,
    team_id: Option<String>,
    assignee_id: Option<String>,
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
pub fn create_issues_from_file_or_dir(token: String, path: String) -> Result<String, String> {
    if Path::is_dir(Path::new(&path)) {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(is_issue_toml)
        {
            create_issues(token.clone(), entry.path().to_str().unwrap().to_string())?;
        }
        Ok("Done".to_string())
    } else {
        create_issues(token, path)
    }
}

fn create_issues(token: String, path: String) -> Result<String, String> {
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
    let team_id = parent.team_id;
    let assignee_id = parent.assignee_id.unwrap_or_default();
    let project_id = parent.project_id;
    let description_template = parent.description.unwrap_or_default();
    let description = fill_in_variables(description_template, variables.clone())?;

    let mut gql_variables = HashMap::new();
    gql_variables.insert("title".to_string(), title);
    gql_variables.insert("teamId".to_string(), team_id.clone());
    gql_variables.insert("assigneeId".to_string(), assignee_id.clone());
    if let Some(id) = project_id {
        gql_variables.insert("projectId".to_string(), id);
    }
    gql_variables.insert("description".to_string(), description);

    let response = request::gql(token.clone(), ISSUE_CREATE_DOC, gql_variables)?;
    let Issue { id, url } = extract_id_from_response(response)?;

    println!("- [{}] {}", id, url);

    for child in children.iter() {
        let title = fill_in_variables(child.title.clone(), variables.clone())?;
        let child_a_id = child
            .assignee_id
            .clone()
            .unwrap_or_else(|| assignee_id.clone());
        let child_team_id = child.team_id.clone().unwrap_or_else(|| team_id.clone());
        let child_description_template = child.description.clone().unwrap_or_default();
        let child_description = fill_in_variables(child_description_template, variables.clone())?;

        let mut gql_variables = HashMap::new();
        gql_variables.insert("title".to_string(), title);
        gql_variables.insert("teamId".to_string(), child_team_id);
        gql_variables.insert("parentId".to_string(), id.clone());
        gql_variables.insert("assigneeId".to_string(), child_a_id);
        gql_variables.insert("description".to_string(), child_description);
        let response = request::gql(token.clone(), ISSUE_CREATE_DOC, gql_variables)?.to_string();
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
