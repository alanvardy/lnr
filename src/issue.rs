use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, fmt::Display};

use crate::{color, config::Config, input, request, viewer};

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
                        branchName
                        state {
                            id
                            name
                            }
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
                        branchName
                        state {
                            id
                            name
                        }
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
                        branchName
                        children {
                            nodes {
                                id
                                identifier
                                title
                                description
                                url
                                branchName
                                state {
                                    id
                                    name
                                }
                            }
                        }
                        state {
                            id
                            name
                        }
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
                        branchName
                        description
                        state {
                            id
                            name
                        }
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
#[allow(non_snake_case)]
struct Issue {
    id: String,
    state: State,
    identifier: String,
    url: String,
    title: String,
    branchName: String,
    description: Option<String>,
    children: Option<IssueListIssues>,
}

#[derive(Deserialize, Serialize, Debug)]
struct State {
    id: String,
    name: String,
}

enum Format {
    View,
    List,
}

impl Issue {
    fn format(&self, format: Format) -> String {
        let title = color::green_string(&self.title);
        let id = &self.identifier;
        let description = self
            .description
            .clone()
            .unwrap_or_else(|| String::from("<No description>"));
        let state = &self.state.name;
        let branch_name = &self.branchName;

        let child_tickets = if self.is_parent() {
            let child_count = self.child_count();
            format!(" | {child_count} child tickets")
        } else {
            String::new()
        };

        match format {
            Format::View => {
                format!("{title}\n{id} | {state}{child_tickets}\n{branch_name}\n\n{description}")
            }

            Format::List => {
                let id = format!("{: >10}", id);
                format!("- {id} | {title}\n             | {state}{child_tickets}\n")
            }
        }
    }

    pub fn is_parent(&self) -> bool {
        self.child_count() > 0
    }

    pub fn child_count(&self) -> u8 {
        match &self.children {
            None => 0,
            Some(IssueListIssues { nodes: issues }) => {
                if issues.is_empty() {
                    0
                } else {
                    issues.len() as u8
                }
            }
        }
    }

    pub fn sort(&self) -> String {
        let parent = if self.is_parent() { 0 } else { 1 };
        let name = self.state.name.clone();
        format!("{parent}{name}")
    }
}

impl Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = color::green_string(&self.title);
        let id = self.identifier.clone();
        let state = self.state.name.clone();

        let child_tickets = if self.is_parent() {
            let child_count = self.child_count();
            format!(" | {child_count} child tickets")
        } else {
            String::new()
        };
        let id = format!("{: >10}", id);

        if self.is_parent() {
            write!(
                f,
                "- {id} | {title}\n             | {state}{child_tickets}\n"
            )
        } else {
            write!(f, "- {id} | {title}\n             | {state}\n")
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
    let issues_text = get_issues(config, token, assignee_id, team_id, project_id).map(|i| {
        i.into_iter()
            .map(|j| j.format(Format::List))
            .collect::<Vec<String>>()
            .join("\n")
    })?;
    let title = color::green_string("Issues");
    Ok(format!("\n{title}\n\n{issues_text}"))
}

fn get_issues(
    config: &Config,
    token: &String,
    assignee_id: Option<String>,
    team_id: Option<String>,
    project_id: Option<String>,
) -> Result<Vec<Issue>, String> {
    let mut and_filters = Vec::new();
    if let Some(project_id) = project_id {
        and_filters.push(json!({"project": {"id": {"eq": project_id}}}));
    }

    if let Some(assignee_id) = assignee_id {
        and_filters.push(json!({"assignee": {"id": {"eq": assignee_id}}}));
    }

    if let Some(team_id) = team_id {
        and_filters.push(json!({"team": {"id": {"eq": team_id}}}));
    }

    and_filters.push(json!({"state": {"name": {"neq": "Done"}}}));
    and_filters.push(json!({"state": {"name": {"neq": "Backlog"}}}));
    and_filters.push(json!({"state": {"name": {"neq": "Triage"}}}));
    and_filters.push(json!({"state": {"name": {"neq": "Canceled"}}}));
    and_filters.push(json!({"state": {"name": {"neq": "Closed"}}}));
    and_filters.push(json!({"state": {"name": {"neq": "Merged to Dev"}}}));

    let filter = json!({ "and": and_filters });
    let mut gql_variables = HashMap::new();
    gql_variables.insert("filter".to_string(), filter);

    let response = request::gql(config, token, ISSUE_LIST_DOC, gql_variables)?;
    issue_list_response(response)
}

pub fn view(config: &Config, token: &String, branch: Option<String>) -> Result<String, String> {
    if let Some(branch) = branch {
        let mut gql_variables = HashMap::new();
        gql_variables.insert("branchName".to_string(), Value::String(branch.clone()));

        let response = request::gql(config, token, ISSUE_VIEW_DOC, gql_variables)?;
        let issue = issue_view_response(response, &branch)?;

        Ok(issue.format(Format::View))
    } else {
        let assignee_id = viewer::get_viewer(config, token)?.id;
        let issues = get_issues(config, token, Some(assignee_id), None, None)?;
        let issue = input::select("Select an issue", issues, None)?;
        Ok(issue.format(Format::View))
    }
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
        }) => {
            let mut issues = issues;
            issues.sort_by_key(|i| i.sort());
            Ok(issues)
        }
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
