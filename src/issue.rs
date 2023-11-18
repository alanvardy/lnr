use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, fmt::Display};

use crate::{
    color,
    config::Config,
    input, request,
    team::{Project, State, Team},
    viewer,
};

const ISSUE_CREATE_DOC: &str = "mutation (
                    $title: String!
                    $teamId: String!
                    $stateId: String!
                    $assigneeId: String
                    $description: String,
                    $projectId: String
                ) {
                issueCreate(
                    input: {
                        title: $title
                        teamId: $teamId
                        stateId: $stateId
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
                            position
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

const ISSUE_BRANCH_VIEW_DOC: &str = "query (
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
                        comments {
                            nodes {
                                body
                                createdAt
                                editedAt
                                url
                                user {
                                    displayName
                                }
                                children {
                                    nodes {
                                        body
                                        createdAt
                                        editedAt
                                        url
                                        user {
                                            displayName
                                        }
                                    }
                                }
                            }
                        }
                        state {
                            id
                            name
                            position
                        }
                    }
                }
                ";

const ISSUE_ID_VIEW_DOC: &str = "query (
                    $id: String!
                ) {
                issue(
                    id: $id
                )   {
                        id
                        identifier
                        url
                        title
                        branchName
                        description
                        comments {
                            nodes {
                                body
                                createdAt
                                editedAt
                                url
                                user {
                                    displayName
                                }
                                children {
                                    nodes {
                                        body
                                        createdAt
                                        editedAt
                                        url
                                        user {
                                            displayName
                                        }
                                    }
                                }
                            }
                        }
                        state {
                            id
                            name
                            position
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
struct IssueBranchViewResponse {
    data: Option<IssueBranchViewData>,
}

#[derive(Deserialize, Serialize, Debug)]
struct IssueIdViewResponse {
    data: IssueData,
}

#[derive(Deserialize, Serialize, Debug)]
struct IssueData {
    issue: Option<Issue>,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct IssueBranchViewData {
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
    comments: Option<CommentsConnection>,
}

#[derive(Deserialize, Serialize, Debug)]
struct CommentsConnection {
    nodes: Vec<Comment>,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct Comment {
    body: String,
    createdAt: String,
    editedAt: Option<String>,
    url: String,
    user: User,
    children: Option<CommentsConnection>,
}

impl Comment {
    fn format(&self) -> String {
        let divider = color::green_string("----------------");
        let body = &self.body;
        let user = color::cyan_string(&self.user.displayName);
        let created_at = &self.createdAt;
        format!("\n{body}\n\n- {user} {created_at}\n\n{divider}")
    }
}
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct User {
    displayName: String,
}

enum Format {
    View,
    List,
}

impl Issue {
    fn format(&self, format: Format) -> String {
        let title = color::green_string(&self.title);
        let id = color::blue_string(&self.identifier);
        let description = self
            .description
            .clone()
            .unwrap_or_else(|| String::from("<No description>"));
        let state = &self.state.name;
        let branch_name = &self.branchName;
        let url = &self.url;

        let child_tickets = if self.is_parent() {
            let child_count = self.child_count();
            format!(" | {child_count} child tickets")
        } else {
            String::new()
        };
        let comments = &self.render_comments();

        match format {
            Format::View => {
                let divider = color::green_string("--- COMMENTS ---");
                format!(
                    "{title}\n{id} | {state}{child_tickets}\n{url}\n{branch_name}\n\n{description}\n\n{divider}\n{comments}"
                )
            }

            Format::List => {
                let id = format!("{: >10}", id);
                format!("- {id} | {title}\n             | {state}{child_tickets}\n")
            }
        }
    }

    fn render_comments(&self) -> String {
        match &self.comments {
            Some(CommentsConnection { nodes }) => {
                let mut comment_text = nodes.iter().map(|c| c.format()).collect::<Vec<String>>();

                if comment_text.is_empty() {
                    String::from("\n<No Comments>")
                } else {
                    comment_text.reverse();
                    comment_text.join("\n")
                }
            }
            None => String::new(),
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
    /// This is for rendering in a select
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
                "- {id} | {title}\n               | {state}{child_tickets}\n"
            )
        } else {
            write!(f, "- {id} | {title}\n               | {state}\n")
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub fn create(
    config: &Config,
    token: &str,
    title: String,
    description: String,
    team: Team,
    project: Option<Project>,
    state: State,
    assignee_id: String,
) -> Result<String, String> {
    let response = request::Gql::new(config, token, ISSUE_CREATE_DOC)
        .put_string("title", title)
        .put_string("assigneeId", assignee_id)
        .put_string("teamId", team.id)
        .put_string("stateId", state.id)
        .maybe_put_string("projectId", project.map(|p| p.id))
        .put_string("description", description)
        .run()?;

    let Issue { url, .. } = issue_create_response(response)?;

    Ok(url)
}

pub fn list(
    config: &Config,
    token: &str,
    assignee_id: Option<String>,
    team: Option<Team>,
    project: Option<Project>,
) -> Result<String, String> {
    let issues_text = get_issues(config, token, assignee_id, team, project).map(|i| {
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
    token: &str,
    assignee_id: Option<String>,
    team: Option<Team>,
    project: Option<Project>,
) -> Result<Vec<Issue>, String> {
    let mut and_filters = Vec::new();
    if let Some(Project { id, .. }) = project {
        and_filters.push(json!({"project": {"id": {"eq": id}}}));
    }

    if let Some(assignee_id) = assignee_id {
        and_filters.push(json!({"assignee": {"id": {"eq": assignee_id}}}));
    }

    if let Some(Team { id, .. }) = team {
        and_filters.push(json!({"team": {"id": {"eq": id}}}));
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

    let response = request::Gql::new(config, token, ISSUE_LIST_DOC)
        .put_variables(gql_variables)
        .run()?;
    issue_list_response(response)
}

pub fn view(config: &Config, token: &str, branch: Option<String>) -> Result<String, String> {
    if let Some(branch) = branch {
        let response = request::Gql::new(config, token, ISSUE_BRANCH_VIEW_DOC)
            .put_string("branchName", branch.clone())
            .run()?;
        let issue = issue_branch_view_response(response, &branch)?;

        Ok(issue.format(Format::View))
    } else {
        let assignee_id = viewer::get_viewer(config, token)?.id;
        let mut issues = get_issues(config, token, Some(assignee_id), None, None)?;
        issues.reverse();
        let issue = input::select("Select an issue", issues, None)?;
        // Need to refetch to get comments

        let response = request::Gql::new(config, token, ISSUE_ID_VIEW_DOC)
            .put_string("id", issue.id)
            .run()?;

        let issue = issue_id_view_response(response)?;
        Ok(issue.format(Format::View))
    }
}

pub fn edit(config: &Config, token: &str, branch: String) -> Result<String, String> {
    let response = request::Gql::new(config, token, ISSUE_BRANCH_VIEW_DOC)
        .put_string("branchName", branch.clone())
        .run()?;
    let issue = issue_branch_view_response(response, &branch)?;
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
    let response = request::Gql::new(config, token, ISSUE_UPDATE_DOC)
        .put_variables(gql_variables)
        .run()?;
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

fn issue_branch_view_response(response: String, branch: &String) -> Result<Issue, String> {
    let data: Result<IssueBranchViewResponse, _> = serde_json::from_str(&response);
    match data {
        Ok(IssueBranchViewResponse {
            data:
                Some(IssueBranchViewData {
                    issueVcsBranchSearch: Some(issue),
                }),
        }) => Ok(issue),
        Ok(IssueBranchViewResponse {
            data:
                Some(IssueBranchViewData {
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

fn issue_id_view_response(response: String) -> Result<Issue, String> {
    let data: Result<IssueIdViewResponse, _> = serde_json::from_str(&response);

    match data {
        Ok(IssueIdViewResponse {
            data: IssueData { issue: Some(issue) },
        }) => Ok(issue),
        Ok(IssueIdViewResponse {
            data: IssueData { issue: None },
        }) => Err(String::from("Issue not found")),
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_create() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::issue_create())
            .create();
        let config = test::fixtures::config().mock_url(server.url());

        let token = "1234";
        let title = "Test".to_string();
        let description = "A Description".to_string();
        let team = test::fixtures::team();
        let state = test::fixtures::state();
        let project = None;
        let assignee_id = "456".to_string();

        let result = create(
            &config,
            token,
            title,
            description,
            team,
            project,
            state,
            assignee_id,
        );
        assert_eq!(
            result,
            Ok("https://linear.app/vardy/issue/BE-3354/test".to_string())
        );
        mock.assert();
    }

    #[test]
    fn test_list() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test::responses::issue_list())
            .create();
        let config = test::fixtures::config().mock_url(server.url());

        let token = "1234";
        let team_id = None;
        let project_id = None;
        let assignee_id = None;

        let result = list(&config, token, assignee_id, team_id, project_id);
        assert_eq!(
            result,
            Ok("\nIssues\n\n-   SHO-2148 | Modify schema\n             | Todo\n".to_string())
        );
        mock.assert();
    }
}
