extern crate clap;
#[cfg(test)]
extern crate matches;

mod color;
mod config;
mod git;
mod input;
mod issue;
mod request;
mod test;
mod viewer;

use clap::{Arg, ArgMatches, Command};
use colored::*;
use config::Config;
use viewer::{Project, Team};

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
            Some(("list", m)) => issue_list(m),
            _ => unreachable!(),
        },
        Some(("org", issue_matches)) => match issue_matches.subcommand() {
            Some(("add", m)) => organization_add(m),
            Some(("remove", m)) => organization_remove(m),
            Some(("list", m)) => organization_list(m),
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
                        .arg(title_arg())
                        .arg(team_arg())
                        .arg(org_arg())
                        .arg(description_arg())
                         .arg(flag_arg("noproject", 'n',  "Do not prompt for a project"))
                        .arg(config_arg()),
                    Command::new("edit")
                        .about("Edit the issue for current branch")
                        .arg(org_arg())
                        .arg(config_arg()),
                    Command::new("list")
                        .about("List issues, maximum of 50. Returns issues assigned to user that are Todo or In Progress")
                        .arg(org_arg())
                        .arg(config_arg()),
                    Command::new("view")
                        .about("View the issue for current branch")
                         .arg(flag_arg("select", 's',  "Select ticket from list view"))
                        .arg(org_arg())
                        .arg(config_arg()),
                ]),
            Command::new("org")
                .arg_required_else_help(true)
                .propagate_version(true)
                .subcommand_required(true)
                .subcommands([
                    Command::new("add")
                        .about("Add an organization and token to config")
                        .arg(config_arg()),
                    Command::new("remove")
                        .about("Remove an organization and token from config")
                        .arg(config_arg()),
                    Command::new("list")
                        .about("List organizations and tokens in config")
                        .arg(config_arg()),
                ]),
        ])
}

#[cfg(not(tarpaulin_include))]
fn issue_create(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let token = fetch_token(matches, &config)?;
    let viewer = viewer::get_viewer(&config, &token)?;
    let team_name = fetch_team_name(matches);
    let team = viewer::team(&viewer, team_name)?;
    let project_id = match has_flag(matches, "noproject") {
        true => None,
        false => get_project(&team)?.map(|p| p.id),
    };
    let title = fetch_string(matches, &config, "title", "Title")?;
    let description = fetch_editor(matches, &config, "description", "Description")?;

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
fn issue_view(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let token = fetch_token(matches, &config)?;
    if has_flag(matches, "select") {
        issue::view(&config, &token, None)
    } else {
        let branch = git::get_branch()?;
        issue::view(&config, &token, Some(branch))
    }
}

#[cfg(not(tarpaulin_include))]
fn issue_edit(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let token = fetch_token(matches, &config)?;

    let branch = git::get_branch()?;
    issue::edit(&config, &token, branch)
}

#[cfg(not(tarpaulin_include))]
fn issue_list(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
    let token = fetch_token(matches, &config)?;

    let viewer = viewer::get_viewer(&config, &token)?;

    // let team = viewer::team(&viewer)?;
    // let project_id = get_project(&team)?.map(|p| p.id);

    issue::list(&config, &token, Some(viewer.id), None, None)
}

fn get_project(team: &Team) -> Result<Option<Project>, String> {
    let mut project_names = viewer::project_names(team)?;
    project_names.sort();
    project_names.insert(0, String::from("None"));
    let project_name = input::select("Select project", project_names, None)?;
    viewer::project(team, project_name)
}

#[cfg(not(tarpaulin_include))]
fn organization_add(matches: &ArgMatches) -> Result<String, String> {
    let mut config = fetch_config(matches)?;
    let name = input::string("Input organization name", None)?;
    let token = input::string("Input organization token", None)?;
    config.add_organization(name, token);
    config.save()
}

#[cfg(not(tarpaulin_include))]
fn organization_list(matches: &ArgMatches) -> Result<String, String> {
    let config = fetch_config(matches)?;
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
fn organization_remove(matches: &ArgMatches) -> Result<String, String> {
    let mut config = fetch_config(matches)?;
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

// --- VALUE HELPERS ---

#[cfg(not(tarpaulin_include))]
fn fetch_config(matches: &ArgMatches) -> Result<Config, String> {
    check_for_latest_version();
    let config_path = matches.get_one::<String>("config").map(|s| s.to_owned());

    config::get_or_create(config_path)
}

#[cfg(not(tarpaulin_include))]
fn fetch_token(matches: &ArgMatches, config: &Config) -> Result<String, String> {
    let org_name = matches.get_one::<String>("org").map(|s| s.to_owned());
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

/// Checks if the flag was used
#[cfg(not(tarpaulin_include))]
fn has_flag(matches: &ArgMatches, id: &'static str) -> bool {
    matches.get_one::<String>(id) == Some(&String::from("yes"))
}
#[cfg(not(tarpaulin_include))]
fn config_arg() -> Arg {
    Arg::new("config")
        .short('c')
        .long("config")
        .num_args(1)
        .required(false)
        .value_name("CONFIGURATION PATH")
        .help("Absolute path of configuration. Defaults to $XDG_CONFIG_HOME/lnr.cfg")
}

#[cfg(not(tarpaulin_include))]
fn org_arg() -> Arg {
    Arg::new("org")
        .short('o')
        .long("org")
        .num_args(1)
        .required(false)
        .value_name("organization name")
        .help("You will be promped at runtime if this isn't provided")
}

#[cfg(not(tarpaulin_include))]
fn title_arg() -> Arg {
    Arg::new("title")
        .short('t')
        .long("title")
        .num_args(1)
        .required(false)
        .value_name("TITLE TEXT")
        .help("Title for issue")
}

#[cfg(not(tarpaulin_include))]
fn team_arg() -> Arg {
    Arg::new("team")
        .short('e')
        .long("team")
        .num_args(1)
        .required(false)
        .value_name("TEAM NAME")
        .help("Team name")
}

#[cfg(not(tarpaulin_include))]
fn description_arg() -> Arg {
    Arg::new("description")
        .short('d')
        .long("description")
        .num_args(1)
        .required(false)
        .value_name("DESCRIPTION TEXT")
        .help("Description for issue")
}

#[cfg(not(tarpaulin_include))]
fn flag_arg(id: &'static str, short: char, help: &'static str) -> Arg {
    Arg::new(id)
        .short(short)
        .long(id)
        .value_parser(["yes", "no"])
        .num_args(0..1)
        .default_value("no")
        .default_missing_value("yes")
        .required(false)
        .help(help)
}

#[cfg(not(tarpaulin_include))]
fn fetch_string(
    matches: &ArgMatches,
    config: &Config,
    field: &str,
    prompt: &str,
) -> Result<String, String> {
    let argument_content = matches.get_one::<String>(field).map(|s| s.to_owned());
    match argument_content {
        Some(string) => Ok(string),
        None => input::string(prompt, config.mock_string.clone()),
    }
}

#[cfg(not(tarpaulin_include))]
fn fetch_editor(
    matches: &ArgMatches,
    config: &Config,
    field: &str,
    prompt: &str,
) -> Result<String, String> {
    let argument_content = matches.get_one::<String>(field).map(|s| s.to_owned());
    match argument_content {
        Some(string) => Ok(string),
        None => input::editor(prompt, "", config.mock_string.clone()),
    }
}

#[cfg(not(tarpaulin_include))]
fn fetch_team_name(matches: &ArgMatches) -> Option<String> {
    matches.get_one::<String>("team").map(|s| s.to_owned())
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
