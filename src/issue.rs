use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{color, config::Config, input, request};

const ISSUE_CREATE_DOC: &str = "mutation (
                    $title: String!
                    $teamId: String!
                    $assigneeId: String
                    $description: String,
                    $projectId: String
                ) {
                issueCreate(
                    input: {
                        title: $title
                        teamId: $teamId
                        assigneeId: $assigneeId
                        description: $description
                        projectId: $projectId
                    }
                ) {
                    issue {
                        id
                        title
                        description
                        url
                    }
                    }
                }
                ";

const ISSUE_UPDATE_DOC: &str = "mutation (
                    $id: String!
                    $input: IssueUpdateInput!,
                ) {
                issueUpdate(
                    id: $id,
                    input: $input
                ) {
                    issue {
                        id
                        title
                        description
                        url
                    }
                    }
                }
                ";

const ISSUE_VIEW_DOC: &str = "query (
                    $branchName: String!
                ) {
                issueVcsBranchSearch(
                    branchName: $branchName
                )   {
                        id
                        url
                        title
                        description
                    }
                }
                ";

// ISSUE VIEW
#[derive(Deserialize, Serialize, Debug)]
struct IssueViewResponse {
    data: Option<IssueViewData>,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct IssueViewData {
    issueVcsBranchSearch: Option<Issue>,
}

// ISSUE CREATE

#[derive(Deserialize, Serialize, Debug)]
struct IssueCreateResponse {
    data: Option<IssueCreateData>,
}
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct IssueCreateData {
    issueCreate: IssueCreate,
}

#[derive(Deserialize, Serialize, Debug)]
struct IssueCreate {
    issue: Issue,
}

// ISSUE UPDATE

#[derive(Deserialize, Serialize, Debug)]
struct IssueUpdateResponse {
    data: Option<IssueUpdateData>,
}
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct IssueUpdateData {
    issueUpdate: IssueUpdate,
}

#[derive(Deserialize, Serialize, Debug)]
struct IssueUpdate {
    issue: Option<Issue>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Issue {
    id: String,
    url: String,
    title: String,
    description: Option<String>,
}

impl Issue {
    fn format(&self) -> String {
        let title = color::blue_string(&self.title);
        let description = &self.description.clone().unwrap_or_default();

        format!("{title}\n\n{description}")
    }
}

pub fn create(
    config: &Config,
    token: String,
    title: String,
    description: String,
    team_id: String,
    project_id: Option<String>,
    assignee_id: String,
) -> Result<String, String> {
    let mut gql_variables = HashMap::new();
    gql_variables.insert("title".to_string(), Value::String(title));
    gql_variables.insert("teamId".to_string(), Value::String(team_id));
    gql_variables.insert("assigneeId".to_string(), Value::String(assignee_id));
    if let Some(id) = project_id {
        gql_variables.insert("projectId".to_string(), Value::String(id));
    }
    gql_variables.insert("description".to_string(), Value::String(description));

    let response = request::gql(config, &token, ISSUE_CREATE_DOC, gql_variables)?;
    let Issue { url, .. } = issue_create_response(response)?;

    Ok(url)
}

pub fn view(config: &Config, token: &String, branch: String) -> Result<String, String> {
    let mut gql_variables = HashMap::new();
    gql_variables.insert("branchName".to_string(), Value::String(branch.clone()));

    let response = request::gql(config, token, ISSUE_VIEW_DOC, gql_variables)?;
    let issue = issue_view_response(response, &branch)?;

    Ok(issue.format())
}

pub fn edit(config: &Config, token: &String, branch: String) -> Result<String, String> {
    let mut gql_variables = HashMap::new();
    gql_variables.insert("branchName".to_string(), Value::String(branch.clone()));

    let response = request::gql(config, token, ISSUE_VIEW_DOC, gql_variables)?;
    let issue = issue_view_response(response, &branch)?;
    // Stops wierd spinner output from rolling into the input text
    println!();
    let description = input::editor(
        "Enter updated description",
        &issue.description.unwrap_or_default(),
        config.mock_string.clone(),
    )?;
    let mut input = HashMap::new();
    input.insert("description".to_string(), description);

    let mut gql_variables = HashMap::new();
    gql_variables.insert("id".to_string(), Value::String(issue.id));
    gql_variables.insert("input".to_string(), json!(input));

    let response = request::gql(config, token, ISSUE_UPDATE_DOC, gql_variables)?;
    let issue = issue_update_response(response)?;
    Ok(issue.url)
}

/// Get the id from an issue response, needed for parent issues and terminal output
fn issue_create_response(response: String) -> Result<Issue, String> {
    let data: Result<IssueCreateResponse, _> = serde_json::from_str(&response);

    match data {
        Ok(IssueCreateResponse {
            data:
                Some(IssueCreateData {
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

fn issue_view_response(response: String, branch: &String) -> Result<Issue, String> {
    let data: Result<IssueViewResponse, _> = serde_json::from_str(&response);

    match data {
        Ok(IssueViewResponse {
            data:
                Some(IssueViewData {
                    issueVcsBranchSearch: Some(issue),
                }),
        }) => Ok(issue),
        Ok(IssueViewResponse {
            data: Some(IssueViewData {
                issueVcsBranchSearch: None,
            }),
        }) => Err(format!("Branch {branch} not found")),
        err => Err(format!(
            "Could not parse response for issue:
            ---
            {err:?}
            ---
            {response:?}"
        )),
    }
}

fn issue_update_response(response: String) -> Result<Issue, String> {
    let data: Result<IssueUpdateResponse, _> = serde_json::from_str(&response);

    match data {
        Ok(IssueUpdateResponse {
            data:
                Some(IssueUpdateData {
                    issueUpdate: IssueUpdate { issue: Some(issue) },
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
