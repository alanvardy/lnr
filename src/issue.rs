use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::request;

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

pub fn create(
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

    let response = request::gql(&token, ISSUE_CREATE_DOC, gql_variables)?;
    let Issue { url, .. } = extract_id_from_response(response)?;

    Ok(url)
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
