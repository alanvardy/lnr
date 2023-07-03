use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{color, config::Config, request};

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
                        descriptionData
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

#[derive(Deserialize, Serialize, Debug)]
struct Issue {
    id: String,
    url: String,
    title: String,
    description: Option<String>,
    description_data: Option<String>,
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
    gql_variables.insert("title".to_string(), title);
    gql_variables.insert("teamId".to_string(), team_id);
    gql_variables.insert("assigneeId".to_string(), assignee_id);
    if let Some(id) = project_id {
        gql_variables.insert("projectId".to_string(), id);
    }
    gql_variables.insert("description".to_string(), description);

    let response = request::gql(config, &token, ISSUE_CREATE_DOC, gql_variables)?;
    let Issue { url, .. } = issue_create_response(response)?;

    Ok(url)
}

pub fn view(config: &Config, token: &String, branch: String) -> Result<String, String> {
    let mut gql_variables = HashMap::new();
    gql_variables.insert("branchName".to_string(), branch.clone());

    let response = request::gql(config, token, ISSUE_VIEW_DOC, gql_variables)?;
    let issue = issue_view_response(response, branch.trim().to_string())?;

    Ok(issue.format())
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

fn issue_view_response(response: String, branch: String) -> Result<Issue, String> {
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
