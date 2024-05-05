extern crate clap;
#[cfg(test)]
extern crate matches;

mod color;
mod config;
mod git;
mod input;
mod issue;
mod priority;
mod request;
mod team;
mod template;
mod test;
mod viewer;

use clap::{Parser, Subcommand};
use colored::*;
use config::Config;
use priority::Priority;
use team::{Project, State, Team};

const NAME: &str = "lnr";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "Alan Vardy <alan@vardy.cc>";
const ABOUT: &str = "A tiny unofficial Linear client";

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
struct OrgAdd {}

#[derive(Parser, Debug, Clone)]
struct OrgRemove {}

#[derive(Parser, Debug, Clone)]
struct OrgList {}

#[derive(Subcommand, Debug, Clone)]
enum TemplateCommands {
    #[clap(alias = "e")]
    /// (e) Create issues from a TOML file
    Evaluate(TemplateEvaluate),
}

#[derive(Parser, Debug, Clone)]
struct TemplateEvaluate {
    #[arg(short, long)]
    /// Path to file or directory
    path: Option<String>,

    #[arg(short = 'e', long)]
    /// Team name
    team: Option<String>,

    #[arg(short, long, default_value_t = false)]
    /// Do not prompt for a project
    noproject: bool,
}

#[derive(Parser, Debug, Clone)]
struct IssueCreate {
    #[arg(short, long)]
    /// Title for issue
    title: Option<String>,

    #[arg(short, long)]
    /// Description for issue
    description: Option<String>,

    #[arg(short = 'e', long)]
    /// Team name
    team: Option<String>,

    #[arg(short, long, default_value_t = false)]
    /// Do not prompt for a project
    noproject: bool,
}

#[derive(Parser, Debug, Clone)]
struct IssueEdit {}

#[derive(Parser, Debug, Clone)]
struct IssueView {
    #[arg(short, long, default_value_t = false)]
    /// Select ticket from list view
    select: bool,
}

#[derive(Parser, Debug, Clone)]
struct IssueList {
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

#[cfg(not(tarpaulin_include))]
fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Issue(IssueCommands::Create(args)) => issue_create(cli.clone(), args),
        Commands::Issue(IssueCommands::Edit(args)) => issue_edit(cli.clone(), args),
        Commands::Issue(IssueCommands::View(args)) => issue_view(cli.clone(), args),
        Commands::Issue(IssueCommands::List(args)) => issue_list(cli.clone(), args),

        Commands::Org(OrgCommands::Add(args)) => org_add(cli.clone(), args),
        Commands::Org(OrgCommands::Remove(args)) => org_remove(cli.clone(), args),
        Commands::Org(OrgCommands::List(args)) => org_list(cli.clone(), args),

        Commands::Template(TemplateCommands::Evaluate(args)) => {
            template_evaluate(cli.clone(), args)
        }
    };

    match result {
        Ok(text) => {
            println!("{text}");
            std::process::exit(0);
        }
        Err(e) => {
            println!("{}", e.red());
            std::process::exit(1);
        }
    }
}

// --- ISSUES ---

#[cfg(not(tarpaulin_include))]
fn issue_create(cli: Cli, args: &IssueCreate) -> Result<String, String> {
    let IssueCreate {
        title,
        description,
        team,
        noproject,
    } = args;
    let config = fetch_config(&cli)?;
    let token = fetch_token(&cli, &config)?;
    let viewer = viewer::get_viewer(&config, &token)?;
    let team = viewer::team(&viewer, team)?;
    let state = get_state(&config, &token, &team)?;
    let priority = get_priority()?;
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

#[cfg(not(tarpaulin_include))]
fn issue_view(cli: Cli, args: &IssueView) -> Result<String, String> {
    let IssueView { select } = args;
    let config = fetch_config(&cli)?;
    let token = fetch_token(&cli, &config)?;
    if *select {
        issue::view(&config, &token, None)
    } else {
        let branch = git::get_branch()?;
        issue::view(&config, &token, Some(branch))
    }
}

#[cfg(not(tarpaulin_include))]
fn issue_edit(cli: Cli, _args: &IssueEdit) -> Result<String, String> {
    let config = fetch_config(&cli)?;
    let token = fetch_token(&cli, &config)?;

    let branch = git::get_branch()?;
    issue::edit(&config, &token, branch)
}

#[cfg(not(tarpaulin_include))]
fn issue_list(cli: Cli, args: &IssueList) -> Result<String, String> {
    let IssueList {
        team,
        noteam,
        noproject,
    } = args;
    let config = fetch_config(&cli)?;
    let token = fetch_token(&cli, &config)?;

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

#[cfg(not(tarpaulin_include))]
fn org_add(cli: Cli, _args: &OrgAdd) -> Result<String, String> {
    let mut config = fetch_config(&cli)?;
    let name = input::string("Input organization name", None)?;
    let token = input::string("Input organization token", None)?;
    config.add_organization(name, token);
    config.save()
}

#[cfg(not(tarpaulin_include))]
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

#[cfg(not(tarpaulin_include))]
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

#[cfg(not(tarpaulin_include))]
fn template_evaluate(cli: Cli, args: &TemplateEvaluate) -> Result<String, String> {
    let TemplateEvaluate {
        path,
        team,
        noproject,
    } = args;
    let config = fetch_config(&cli)?;
    let token = fetch_token(&cli, &config)?;
    let viewer = viewer::get_viewer(&config, &token)?;
    let team = viewer::team(&viewer, team)?;
    let priority = get_priority()?;
    let state = get_state(&config, &token, &team)?;
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

#[cfg(not(tarpaulin_include))]
fn fetch_config(cli: &Cli) -> Result<Config, String> {
    check_for_latest_version();
    config::get_or_create(cli.config.clone())
}

#[cfg(not(tarpaulin_include))]
fn fetch_token(cli: &Cli, config: &Config) -> Result<String, String> {
    let org_name = cli.org.clone();
    match org_name {
        Some(string) => config.token(&string),
        None => {
            let mut org_names = config.organization_names();
            org_names.sort();

            if org_names.is_empty() {
                let command = color::cyan_string("org add");
                Err(format!("Add an organization with {}", command))
            } else if org_names.len() == 1 {
                config.token(org_names.first().unwrap())
            } else {
                let org_name = input::select("Select an organization", org_names, None)?;
                config.token(&org_name)
            }
        }
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

fn get_state(config: &Config, token: &str, team: &Team) -> Result<State, String> {
    let states = team::get_states(config, token, team)?;
    input::select("Select state", states, None)
}

fn get_priority() -> Result<Priority, String> {
    let priorities = priority::all_priorities();
    input::select("Select priority", priorities, None)
}

// #[cfg(not(tarpaulin_include))]
// fn path_arg() -> Arg {
//     Arg::new("path")
//         .short('p')
//         .long("path")
//         .num_args(1)
//         .required(false)
//         .value_name("PATH")
//         .help("Path to file or directory")
// }

#[cfg(not(tarpaulin_include))]
fn fetch_string(value: &Option<String>, config: &Config, prompt: &str) -> Result<String, String> {
    match value {
        Some(string) => Ok(string.to_owned()),
        None => input::string(prompt, config.mock_string.clone()),
    }
}

#[cfg(not(tarpaulin_include))]
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
