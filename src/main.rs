extern crate clap;
#[cfg(test)]
extern crate matches;

mod request;
use clap::{Arg, ArgMatches, Command};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const APP: &str = "lnr";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "Alan Vardy <alan@vardy.cc>";
const ABOUT: &str = "A tiny unofficial Linear client";

const FETCH_IDS_DOC: &str = "
        query {
            viewer {
                name
                id
                teamMemberships {
                    nodes {
                        team {
                            name
                            id
                            projects {
                                nodes {
                                    name
                                    id
                                }
                            }
                        }
                    }
                }
            }
        }";

#[derive(Serialize, Deserialize, Debug)]
struct ViewerData {
    data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    viewer: Viewer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Viewer {
    id: String,
    name: String,
    team_memberships: TeamMemberships,
}

#[derive(Serialize, Deserialize, Debug)]
struct TeamMemberships {
    nodes: Vec<TeamNode>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TeamNode {
    team: Team,
}

#[derive(Serialize, Deserialize, Debug)]
struct Team {
    name: String,
    id: String,
}

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
    dbg!(get_viewer(token).unwrap());

    Ok(String::from("ok"))
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
// TODO move this to viewer.rs
fn get_viewer(token: String) -> Result<Viewer, String> {
    let json = request::gql(token, FETCH_IDS_DOC, HashMap::new())?;

    let result: Result<ViewerData, _> = serde_json::from_str(&json);
    match result {
        Ok(body) => Ok(body.data.viewer),
        Err(err) => Err(format!("Could not parse response for item: {err:?}")),
    }
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
