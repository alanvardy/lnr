extern crate clap;
#[cfg(test)]
extern crate matches;

mod color;
mod config;
mod input;
mod issue;
mod request;
mod test;
mod viewer;

use clap::{Arg, ArgMatches, Command};
use colored::*;
use config::Config;

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
            Some(("view", m)) => issue_view(m),
            Some(("edit", m)) => issue_edit(m),
            _ => unreachable!(),
        },
        Some(("org", issue_matches)) => match issue_matches.subcommand() {
            Some(("add", m)) => organization_add(m),
            Some(("remove", m)) => organization_remove(m),
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
                .subcommands([
                    Command::new("create")
                        .about("Create a new issue")
                        .arg(config_arg()),
                    Command::new("edit")
                        .about("Edit the issue for current branch")
                        .arg(config_arg()),
                    Command::new("view")
                        .about("View the issue for current branch")
                        .arg(config_arg()),
                ]),
            Command::new("org")
                .arg_required_else_help(true)
                .propagate_version(true)
                .subcommand_required(true)
                .subcommands([
                    Command::new("add")
                        .about("Add an organization and token to config")
                        .arg(config_arg())
                        .arg(name_arg())
                        .arg(token_arg()),
                    Command::new("remove")
                        .about("Remove an organization and token from config")
                        .arg(config_arg())
                        .arg(name_arg()),
                ]),
        ])
}

#[cfg(not(tarpaulin_include))]
fn issue_create(_matches: &ArgMatches) -> Result<String, String> {
    check_for_latest_version();
    let config = config::get_or_create(None)?;
    let token = get_token(&config)?;
    let viewer = viewer::get_viewer(&config, &token)?;

    let team = viewer::team(&viewer)?;
    let mut project_names = viewer::project_names(&team)?;
    project_names.push(String::from("None"));
    let project_name = input::select("Select project", project_names, None)?;
    let project = viewer::project(&team, project_name)?;
    let title = input::string("Enter title", None)?;
    let description = input::editor("Enter description", "", None)?;
    let project_id = project.map(|p| p.id);

    issue::create(
        &config,
        token,
        title,
        description,
        team.id,
        project_id,
        viewer.id,
    )
}

#[cfg(not(tarpaulin_include))]
fn issue_view(_matches: &ArgMatches) -> Result<String, String> {
    check_for_latest_version();
    let config = config::get_or_create(None)?;
    let token = get_token(&config)?;

    let branch = get_git_branch()?;
    issue::view(&config, &token, branch)
}

#[cfg(not(tarpaulin_include))]
fn issue_edit(_matches: &ArgMatches) -> Result<String, String> {
    check_for_latest_version();
    let config = config::get_or_create(None)?;
    let token = get_token(&config)?;

    let branch = get_git_branch()?;
    issue::edit(&config, &token, branch)
}

fn get_git_branch() -> Result<String, String> {
    let output = std::process::Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|e| e.to_string())
    } else {
        Err(String::from_utf8(output.stderr)).unwrap()
    }
}

fn get_token(config: &Config) -> Result<String, String> {
    let org_names = config.organization_names();
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

#[cfg(not(tarpaulin_include))]
fn organization_add(_matches: &ArgMatches) -> Result<String, String> {
    check_for_latest_version();

    let mut config = config::get_or_create(None)?;
    let name = input::string("Input organization name", None)?;
    let token = input::string("Input organization token", None)?;
    config.add_organization(name, token);
    config.save()
}

#[cfg(not(tarpaulin_include))]
fn organization_remove(_matches: &ArgMatches) -> Result<String, String> {
    check_for_latest_version();

    let mut config = config::get_or_create(None)?;
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
