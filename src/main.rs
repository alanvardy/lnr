extern crate clap;
#[cfg(test)]
extern crate matches;

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
        .subcommands([Command::new("issue")
            .arg_required_else_help(true)
            .propagate_version(true)
            .subcommand_required(true)
            .subcommands([Command::new("create")
                .about("Create a new task")
                .arg(config_arg())
                .arg(content_arg())
                .arg(project_arg())])])
}

#[cfg(not(tarpaulin_include))]
fn issue_create(_matches: &ArgMatches) -> Result<String, String> {
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
fn project_arg() -> Arg {
    Arg::new("project")
        .short('p')
        .long("project")
        .num_args(1)
        .required(false)
        .value_name("PROJECT NAME")
        .help("The project into which the task will be added")
}

#[test]
fn verify_cmd() {
    cmd().debug_assert();
}
