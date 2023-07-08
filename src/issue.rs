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
                        identifier
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
                        identifier
                        title
                        description
                        url
                    }
                    }
                }
                ";

const ISSUE_LIST_DOC: &str = "query (
                    $filter: IssueFilter,
                ) {
                issues (
                    filter: $filter
                ) {
                        nodes {
                            id
                            identifier
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
                        identifier
                        url
                        title
                        description
                    }
                }
                ";

// ISSUE LIST
#[derive(Deserialize, Serialize, Debug)]
struct IssueListResponse {
    data: Option<IssueListData>,
}

#[derive(Deserialize, Serialize, Debug)]
struct IssueListData {
    issues: IssueListIssues,
}

#[derive(Deserialize, Serialize, Debug)]
struct IssueListIssues {
    nodes: Vec<Issue>,
}

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
    identifier: String,
    url: String,
    title: String,
    description: Option<String>,
}

enum Format {
    View,
    List,
}

impl Issue {
    fn format(&self, format: Format) -> String {
        let title = color::blue_string(&self.title);
        let identifier = self.identifier.clone();
        let description = &self.description.clone().unwrap_or_default();

        match format {
            Format::View => format!("{title}\n\n{description}"),
            Format::List => format!("- {identifier} | {title}"),
        }
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

pub fn list(
    config: &Config,
    token: &String,
    assignee_id: Option<String>,
    team_id: Option<String>,
    project_id: Option<String>,
) -> Result<String, String> {
    let mut filter = HashMap::new();
    if let Some(project_id) = project_id {
        let mut id = HashMap::new();
        id.insert("eq".to_string(), Value::String(project_id));
        let mut project = HashMap::new();
        project.insert("id".to_string(), id);
        filter.insert("project".to_string(), project);
    }

    if let Some(assignee_id) = assignee_id {
        let mut id = HashMap::new();
        id.insert("eq".to_string(), Value::String(assignee_id));
        let mut project = HashMap::new();
        project.insert("id".to_string(), id);
        filter.insert("assignee".to_string(), project);
    }

    if let Some(team_id) = team_id {
        let mut id = HashMap::new();
        id.insert("eq".to_string(), Value::String(team_id));
        let mut project = HashMap::new();
        project.insert("id".to_string(), id);
        filter.insert("team".to_string(), project);
    }

    let mut gql_variables = HashMap::new();
    gql_variables.insert("filter".to_string(), json!(filter));

    let response = request::gql(config, token, ISSUE_LIST_DOC, gql_variables)?;
    let issues_text = issue_list_response(response).map(|i| {
        i.into_iter()
            .map(|j| j.format(Format::List))
            .collect::<Vec<String>>()
            .join("\n")
    })?;
    let title = color::green_string("Issues");
    Ok(format!("\n{title}\n\n{issues_text}"))
}

pub fn view(config: &Config, token: &String, branch: String) -> Result<String, String> {
    let mut gql_variables = HashMap::new();
    gql_variables.insert("branchName".to_string(), Value::String(branch.clone()));

    let response = request::gql(config, token, ISSUE_VIEW_DOC, gql_variables)?;
    let issue = issue_view_response(response, &branch)?;

    Ok(issue.format(Format::View))
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

fn issue_list_response(response: String) -> Result<Vec<Issue>, String> {
    let data: Result<IssueListResponse, _> = serde_json::from_str(&response);

    match data {
        Ok(IssueListResponse {
            data:
                Some(IssueListData {
                    issues: IssueListIssues { nodes: issues },
                }),
        }) => Ok(issues),
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
