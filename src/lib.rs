extern crate clap;
#[cfg(test)]
extern crate matches;

// Linear related
mod config;
mod issue;
mod request;
mod team;
mod template;
mod viewer;

// Internal use
mod color;
mod git;
mod input;
mod priority;
mod test;

use clap::{Parser, Subcommand};
use colored::*;
use config::Config;
use priority::Priority;
use team::{Project, State, Team};

const NAME: &str = "lnr";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "Alan Vardy <alan@vardy.cc>";
const ABOUT: &str = "A tiny unofficial Linear client";

/// Wrapper around the inner `Cli` functionality to provide a more ergonomic interface.
#[derive(Clone)]
pub struct LinearClient {
    cli: Cli,
}

impl LinearClient {
    /// Creates a new `LinearClient` instance with the provided configuration path and organization name.
    pub fn new(config_path: Option<String>, org_name: &str) -> Self {
        Self {
            cli: Cli {
                config: config_path,
                org: Some(org_name.to_string()),

                // A dummy command is fine since this should not end up being used.
                command: Commands::Issue(IssueCommands::List(IssueList {
                    team: None,
                    noteam: false,
                    noproject: false,
                })),
            },
        }
    }

    /// Creates a new issue using the provided arguments.
    pub fn issue_create(&self, args: &IssueCreate) -> Result<String, String> {
        issue_create(self.cli.clone(), args)
    }

    /// Views the issue for the current branch, or allows selecting an issue if `select` is true.
    pub fn issue_view(&self, args: &IssueView) -> Result<String, String> {
        issue_view(self.cli.clone(), args)
    }

    /// Edits the issue associated with the current branch.
    pub fn issue_edit(&self, args: &IssueEdit) -> Result<String, String> {
        issue_edit(self.cli.clone(), args)
    }

    /// Lists issues assigned to the user, filtered by the provided arguments.
    pub fn issue_list(&self, args: &IssueList) -> Result<String, String> {
        issue_list(self.cli.clone(), args)
    }

    /// Adds an organization and token to the configuration.
    pub fn org_add(&self, args: &OrgAdd) -> Result<String, String> {
        org_add(self.cli.clone(), args)
    }

    /// Lists all organizations in the configuration.
    pub fn org_list(&self, args: &OrgList) -> Result<String, String> {
        org_list(self.cli.clone(), args)
    }

    /// Removes an organization and token from the configuration.
    pub fn org_remove(&self, args: &OrgRemove) -> Result<String, String> {
        org_remove(self.cli.clone(), args)
    }

    /// Evaluates a template and creates issues based on the provided TOML file or directory.
    pub fn template_evaluate(&self, args: &TemplateEvaluate) -> Result<String, String> {
        template_evaluate(self.cli.clone(), args)
    }

    /// Retrieves the project associated with the provided team.
    pub fn get_project(&self, team: &Option<Team>) -> Result<Option<Project>, String> {
        get_project(team)
    }

    /// Retrieves the state associated with the provided team and state name.
    pub fn get_state(
        &self,
        config: &Config,
        token: &str,
        team: &Team,
        state: &Option<String>,
    ) -> Result<State, String> {
        get_state(config, token, team, state)
    }

    /// Retrieves the priority based on the provided priority level.
    pub fn get_priority(&self, priority: &Option<u8>) -> Result<Priority, String> {
        get_priority(priority)
    }
}

#[derive(Parser, Clone)]
#[command(name = NAME)]
#[command(version = VERSION)]
#[command(about = ABOUT, long_about = None)]
#[command(author = AUTHOR, version)]
#[command(arg_required_else_help(true))]
struct Cli {
    #[arg(short, long)]
    /// Absolute path of configuration. Defaults to $XDG_CONFIG_HOME/lnr.cfg
    config: Option<String>,

    #[arg(short, long)]
    /// You will be prompted at runtime if this isn't provided
    org: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    #[command(subcommand)]
    #[clap(alias = "i")]
    /// (i) Commands for issues
    Issue(IssueCommands),

    #[command(subcommand)]
    #[clap(alias = "o")]
    /// (o) Commands for organizations
    Org(OrgCommands),

    #[command(subcommand)]
    #[clap(alias = "t")]
    /// (t) Commands for working with templates
    Template(TemplateCommands),
}

#[derive(Subcommand, Debug, Clone)]
enum IssueCommands {
    #[clap(alias = "c")]
    /// (c) Create a new issue
    Create(IssueCreate),

    #[clap(alias = "e")]
    /// (e) Edit the issue for current branch
    Edit(IssueEdit),

    #[clap(alias = "v")]
    /// (v) View the issue for current branch
    View(IssueView),

    #[clap(alias = "l")]
    /// (l) List issues, maximum of 50. Returns issues assigned to user that are Todo or In Progress
    List(IssueList),
}

#[derive(Subcommand, Debug, Clone)]
enum OrgCommands {
    #[clap(alias = "a")]
    /// (a) Add an organization and token to config
    Add(OrgAdd),

    #[clap(alias = "r")]
    /// (r) Remove an organization and token from config
    Remove(OrgRemove),

    #[clap(alias = "l")]
    /// (l) List organizations in config
    List(OrgList),
}

#[derive(Parser, Debug, Clone)]
pub struct OrgAdd {}

#[derive(Parser, Debug, Clone)]
pub struct OrgRemove {}

#[derive(Parser, Debug, Clone)]
pub struct OrgList {}

#[derive(Subcommand, Debug, Clone)]
enum TemplateCommands {
    #[clap(alias = "e")]
    /// (e) Create issues from a TOML file
    Evaluate(TemplateEvaluate),
}

#[derive(Parser, Debug, Clone)]
pub struct TemplateEvaluate {
    #[arg(short, long)]
    /// Path to file or directory
    path: Option<String>,

    #[arg(short = 'e', long)]
    /// Team name
    team: Option<String>,

    #[arg(short, long, default_value_t = false)]
    /// Do not prompt for a project
    noproject: bool,

    #[arg(short = 'r', long)]
    /// 1 (Low), 2 (Normal), 3 (High), or 4 (Urgent)
    priority: Option<u8>,

    #[arg(short, long)]
    /// i.e. Backlog or Todo
    state: Option<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct IssueCreate {
    #[arg(short, long)]
    /// Title for issue
    pub title: Option<String>,

    #[arg(short, long)]
    /// Description for issue
    pub description: Option<String>,

    #[arg(short = 'r', long)]
    /// 1 (Low), 2 (Normal), 3 (High), or 4 (Urgent)
    pub priority: Option<u8>,

    #[arg(short = 'e', long)]
    /// Team name
    pub team: Option<String>,

    #[arg(short, long, default_value_t = false)]
    /// Do not prompt for a project
    pub noproject: bool,

    #[arg(short, long)]
    /// i.e. Backlog or Todo
    pub state: Option<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct IssueEdit {}

#[derive(Parser, Debug, Clone)]
pub struct IssueView {
    #[arg(short, long, default_value_t = false)]
    /// Select ticket from list view
    select: bool,
}

#[derive(Parser, Debug, Clone)]
pub struct IssueList {
    #[arg(short = 'e', long)]
    /// Team name
    team: Option<String>,

    #[arg(short, long, default_value_t = false)]
    /// Don't prompt for project
    noproject: bool,

    #[arg(short = 't', long, default_value_t = false)]
    /// Don't prompt for team
    noteam: bool,
}

// --- ISSUES ---

fn issue_create(cli: Cli, args: &IssueCreate) -> Result<String, String> {
    let IssueCreate {
        title,
        description,
        team,
        noproject,
        priority,
        state,
    } = args;
    let config = fetch_config(&cli)?;
    let token = fetch_token()?;
    let viewer = viewer::get_viewer(&config, &token)?;
    let team = viewer::team(&viewer, team)?;
    let state = get_state(&config, &token, &team, state)?;
    let priority = get_priority(priority)?;
    let project = match noproject {
        true => None,
        false => get_project(&Some(team.clone()))?,
    };
    let title = fetch_string(title, &config, "Title")?;
    let description = fetch_editor(description, &config, "Description")?;

    issue::create(
        &config,
        &token,
        title,
        description,
        team,
        project,
        state,
        viewer.id,
        priority,
    )
}

fn issue_view(cli: Cli, args: &IssueView) -> Result<String, String> {
    let IssueView { select } = args;
    let config = fetch_config(&cli)?;
    let token = fetch_token()?;
    if *select {
        issue::view(&config, &token, None)
    } else {
        let branch = git::get_branch()?;
        issue::view(&config, &token, Some(branch))
    }
}

fn issue_edit(cli: Cli, _args: &IssueEdit) -> Result<String, String> {
    let config = fetch_config(&cli)?;
    let token = fetch_token()?;

    let branch = git::get_branch()?;
    issue::edit(&config, &token, branch)
}

fn issue_list(cli: Cli, args: &IssueList) -> Result<String, String> {
    let IssueList {
        team,
        noteam,
        noproject,
    } = args;
    let config = fetch_config(&cli)?;
    let token = fetch_token()?;

    let viewer = viewer::get_viewer(&config, &token)?;

    let team = match *noteam {
        true => None,
        false => Some(viewer::team(&viewer, team)?),
    };
    let project = match *noproject {
        true => None,
        false => get_project(&team)?,
    };

    issue::list(&config, &token, Some(viewer.id), team, project)
}

// --- ORGANIZATIONS ---

fn org_add(cli: Cli, _args: &OrgAdd) -> Result<String, String> {
    let mut config = fetch_config(&cli)?;
    let name = input::string("Input organization name", None)?;
    let token = input::string("Input organization token", None)?;
    config.add_organization(name, token);
    config.save()
}

fn org_list(cli: Cli, _args: &OrgList) -> Result<String, String> {
    let config = fetch_config(&cli)?;
    let orgs = config
        .organizations
        .into_iter()
        .map(|(k, v)| format!("- {k}: {v}"))
        .collect::<Vec<String>>();

    if orgs.is_empty() {
        Ok("No organizations in config".to_string())
    } else {
        let title = color::green_string("Organizations");
        let orgs = orgs.join("\n");

        Ok(format!("{title}\n\n{orgs}"))
    }
}

fn org_remove(cli: Cli, _args: &OrgRemove) -> Result<String, String> {
    let mut config = fetch_config(&cli)?;
    let org_names = config.organization_names();
    if org_names.is_empty() {
        let command = color::cyan_string("org add");
        Err(format!("Add an organization with {}", command))
    } else {
        let org_name = input::select("Select an organization", org_names, None)?;
        config.remove_organization(&org_name);
        config.save()
    }
}

// --- TEMPLATES ---

fn template_evaluate(cli: Cli, args: &TemplateEvaluate) -> Result<String, String> {
    let TemplateEvaluate {
        path,
        team,
        noproject,
        priority,
        state,
    } = args;
    let config = fetch_config(&cli)?;
    let token = fetch_token()?;
    let viewer = viewer::get_viewer(&config, &token)?;
    let team = viewer::team(&viewer, team)?;
    let priority = get_priority(priority)?;
    let state = get_state(&config, &token, &team, state)?;
    let path = fetch_string(path, &config, "Enter path to TOML file or directory")?;
    let project = match *noproject {
        true => None,
        false => get_project(&Some(team.clone()))?,
    };

    template::evaluate(
        &config, &token, &team, &project, &viewer, &path, &state, &priority,
    )
}

// --- VALUE HELPERS ---

fn fetch_config(cli: &Cli) -> Result<Config, String> {
    check_for_latest_version();
    config::get_or_create(cli.config.clone())
}

fn fetch_token() -> Result<String, String> {
    // grab the env variable called LINEAR_API_KEY and return it
    if let Ok(token) = std::env::var("LINEAR_API_KEY") {
        return Ok(token);
    } else {
        Err("LINEAR_API_KEY not found in environment".to_string())
    }
}

fn get_project(team: &Option<Team>) -> Result<Option<Project>, String> {
    let mut project_names = viewer::project_names(team)?;
    if project_names.is_empty() {
        return Ok(None);
    }
    project_names.sort();
    project_names.insert(0, String::from("None"));
    let project_name = input::select("Select project", project_names, None)?;
    viewer::project(team, project_name)
}

fn get_state(
    config: &Config,
    token: &str,
    team: &Team,
    state: &Option<String>,
) -> Result<State, String> {
    let states = team::get_states(config, token, team)?;

    match state {
        None => input::select("Select state", states, None),
        Some(state_name) => {
            let matching_state = states
                .into_iter()
                .filter(|s| s.name == *state_name)
                .collect::<Vec<State>>();

            match matching_state.first() {
                None => Err(format!("{state_name} state not found")),
                Some(state) => Ok(state.to_owned()),
            }
        }
    }
}

fn get_priority(priority: &Option<u8>) -> Result<Priority, String> {
    match priority {
        None => {
            let priorities = priority::all_priorities();
            input::select("Select priority", priorities, None)
        }
        Some(1) => Ok(Priority::Low),
        Some(2) => Ok(Priority::Normal),
        Some(3) => Ok(Priority::High),
        Some(4) => Ok(Priority::Urgent),
        Some(num) => Err(format!(
            "Priority {num} is not valid. Must choose between 1 and 4."
        )),
    }
}

fn fetch_string(value: &Option<String>, config: &Config, prompt: &str) -> Result<String, String> {
    match value {
        Some(string) => Ok(string.to_owned()),
        None => input::string(prompt, config.mock_string.clone()),
    }
}

fn fetch_editor(value: &Option<String>, config: &Config, prompt: &str) -> Result<String, String> {
    match value {
        Some(string) => Ok(string.to_owned()),
        None => input::editor(prompt, "", config.mock_string.clone()),
    }
}

fn check_for_latest_version() {
    match request::get_latest_version() {
        Ok(version) if version.as_str() != VERSION => {
            println!(
                "Latest {} version is {}, found {}.\nRun {} to update if you installed with Cargo",
                NAME,
                version,
                VERSION,
                format!("cargo install {NAME} --force").bright_cyan()
            );
        }
        Ok(_) => (),
        Err(err) => println!(
            "{}, {:?}",
            format!("Could not fetch {NAME} version from Cargo.io").red(),
            err
        ),
    };
}

#[test]
fn verify_cmd() {
    use clap::CommandFactory;
    // Mostly checks that it is not going to throw an exception because of conflicting short arguments
    Cli::try_parse().err();
    Cli::command().debug_assert();
}

#[test]
fn test_api_access() {
    let linear_api = LinearClient::new(None, "Gitar");

    let args = IssueList {
        team: Some("Gitar".to_string()),
        noteam: true,
        noproject: true,
    };

    let issues = linear_api.issue_list(&args).unwrap();

    for issue in issues.lines() {
        println!("{}", issue);
    }

    assert!(true);
}

#[test]
fn test_api_creation() {
    let linear_api = LinearClient::new(None, "Gitar");
    let args = IssueCreate {
        title: Some("Test Issue KAKAPIO".to_string()),
        description: Some("This is a test issue made by Kakapio's code".to_string()),
        priority: Some(1),
        team: Some("Gitar".to_string()),
        noproject: true,
        state: Some("Backlog".to_string()),
    };

    match linear_api.issue_create(&args) {
        Ok(out) => {
            println!("{}", out);
            assert!(true);
        }
        Err(e) => {
            println!("{}", e);
            assert!(false);
        }
    };
}
