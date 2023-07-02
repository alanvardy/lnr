extern crate clap;
#[cfg(test)]
extern crate matches;

mod input;
mod issue;
mod request;
mod viewer;

use clap::{Arg, ArgMatches, Command};
use colored::*;

const APP: &str = "lnr";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "Alan Vardy <alan@vardy.cc>";
const ABOUT: &str = "A tiny unofficial Linear client";

#[cfg(not(tarpaulin_include))]
fn main() {
    let matches = cmd().get_matches();

    let result = match matches.subcommand() {
        Some(("issue", issue_matches)) => match issue_matches.subcommand() {
            Some(("create", m)) => issue_create(m),
            _ => unreachable!(),
        },
        Some(("token", issue_matches)) => match issue_matches.subcommand() {
            Some(("add", m)) => token_add(m),
            _ => unreachable!(),
        },
        _ => unreachable!(),
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

fn cmd() -> Command {
    Command::new(APP)
        .version(VERSION)
        .author(AUTHOR)
        .about(ABOUT)
        .arg_required_else_help(true)
        .propagate_version(true)
        .subcommands([
            Command::new("issue")
                .arg_required_else_help(true)
                .propagate_version(true)
                .subcommand_required(true)
                .subcommands([Command::new("create")
                    .about("Create a new task")
                    .arg(config_arg())
                    .arg(content_arg())
                    .arg(project_arg())]),
            Command::new("token")
                .arg_required_else_help(true)
                .propagate_version(true)
                .subcommand_required(true)
                .subcommands([Command::new("add")
                    .about("Add a token to config")
                    .arg(config_arg())
                    .arg(name_arg())
                    .arg(token_arg())]),
        ])
}

#[cfg(not(tarpaulin_include))]
fn issue_create(_matches: &ArgMatches) -> Result<String, String> {
    check_for_latest_version();
    let token = std::env::var("LINEAR_TOKEN").expect("LINEAR_TOKEN environment variable not set");
    let viewer = viewer::get_viewer(&token)?;
    let team_names = viewer::team_names(&viewer)?;
    let team_name = input::select("Select team", team_names, Some(0))?;

    let team = viewer::team(&viewer, team_name)?;

    let mut project_names = viewer::project_names(&team)?;
    project_names.push(String::from("None"));
    let project_name = input::select("Select project", project_names, Some(0))?;
    let project = viewer::project(&team, project_name)?;
    let title = input::string("Enter title", None)?;
    let description = input::editor("Enter description", None)?;
    let project_id = project.map(|p| p.id);

    issue::create(token, title, description, team.id, project_id, viewer.id)
}

#[cfg(not(tarpaulin_include))]
fn token_add(_matches: &ArgMatches) -> Result<String, String> {
    check_for_latest_version();
    Ok(String::from("ok"))
}

#[cfg(not(tarpaulin_include))]
fn config_arg() -> Arg {
    Arg::new("config")
        .short('o')
        .long("config")
        .num_args(1)
        .required(false)
        .value_name("CONFIGURATION PATH")
        .help("Absolute path of configuration. Defaults to $XDG_CONFIG_HOME/lnr.cfg")
}

#[cfg(not(tarpaulin_include))]
fn content_arg() -> Arg {
    Arg::new("content")
        .short('c')
        .long("content")
        .num_args(1)
        .required(false)
        .value_name("TASK TEXT")
        .help("Content for task")
}

#[cfg(not(tarpaulin_include))]
fn name_arg() -> Arg {
    Arg::new("name")
        .short('n')
        .long("name")
        .num_args(1)
        .required(false)
        .value_name("ORG NAME")
        .help("Name for organization token")
}

#[cfg(not(tarpaulin_include))]
fn token_arg() -> Arg {
    Arg::new("token")
        .short('t')
        .long("token")
        .num_args(1)
        .required(false)
        .value_name("TOKEN")
        .help("Token for organization")
}

#[cfg(not(tarpaulin_include))]
fn project_arg() -> Arg {
    Arg::new("project")
        .short('p')
        .long("project")
        .num_args(1)
        .required(false)
        .value_name("PROJECT NAME")
        .help("The project into which the task will be added")
}
fn check_for_latest_version() {
    match request::get_latest_version() {
        Ok(version) if version.as_str() != VERSION => {
            println!(
                "Latest {} version is {}, found {}.\nRun {} to update if you installed with Cargo",
                APP,
                version,
                VERSION,
                format!("cargo install {APP} --force").bright_cyan()
            );
        }
        Ok(_) => (),
        Err(err) => println!(
            "{}, {:?}",
            format!("Could not fetch {APP} version from Cargo.io").red(),
            err
        ),
    };
}
#[test]
fn verify_cmd() {
    cmd().debug_assert();
}
