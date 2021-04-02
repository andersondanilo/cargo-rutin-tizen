use crate::error::TizenError;
use clap::Arg;
use clap::{App, AppSettings, SubCommand};
use std::{env, process};
mod commands;
mod error;
mod helpers;
mod tizen_env;
use colored::*;
use tizen_env::TizenEnv;

fn main() {
    let tizen_env_args = make_tizen_env_args();
    let forward_args = make_forward_arg();
    let assume_yes_arg = make_assume_yes_arg();
    let release_arg = make_release_arg();

    let app_matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::TrailingVarArg)
        .subcommand(
            SubCommand::with_name("build")
                .about("Wrapper arround cargo build")
                .args(&tizen_env_args)
                .arg(&release_arg)
                .arg(&forward_args),
        )
        .subcommand(
            SubCommand::with_name("package")
                .about("Wrapper arround tizen package")
                .args(&tizen_env_args)
                .arg(&assume_yes_arg)
                .arg(&release_arg)
                .arg(&forward_args),
        )
        .subcommand(
            SubCommand::with_name("install")
                .about("Wrapper arround tizen install")
                .args(&tizen_env_args)
                .arg(&release_arg)
                .arg(&forward_args),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Wrapper arround tizen run")
                .args(&tizen_env_args)
                .arg(&forward_args),
        )
        .subcommand(
            SubCommand::with_name("clean")
                .about("Wrapper arround cargo clean")
                .args(&tizen_env_args)
                .arg(&release_arg)
                .arg(&forward_args),
        )
        .subcommand(
            SubCommand::with_name("config")
                .about("Show config used for building the app")
                .args(&tizen_env_args)
                .arg(&release_arg)
                .arg(
                    Arg::with_name("env_key")
                        .required(false)
                        .takes_value(true)
                        .help("Detail about config key"),
                ),
        )
        .subcommand(
            SubCommand::with_name("dev")
                .about("Build, package, install and run")
                .args(&tizen_env_args)
                .arg(&assume_yes_arg)
                .arg(&release_arg)
                .arg(&forward_args),
        )
        .get_matches_from(get_os_args());

    let tizen_env = match app_matches.subcommand_name() {
        Some(sub_name) => match app_matches.subcommand_matches(sub_name) {
            Some(sub_matches) => {
                match TizenEnv::from_cargo_config(&env::current_dir().unwrap(), &sub_matches) {
                    Ok(obj) => obj,
                    Err(message) => {
                        eprintln!("[ERROR] {}", message);
                        std::process::exit(1)
                    }
                }
            }
            None => {
                eprintln!("No command args matched");
                process::exit(1);
            }
        },
        None => {
            eprintln!("No command matched");
            process::exit(1);
        }
    };

    let command_result = match app_matches.subcommand_name() {
        Some(name @ "config") => {
            commands::config::run(&tizen_env, app_matches.subcommand_matches(&name).unwrap())
        }
        Some(name @ "build") => {
            commands::build::run(&tizen_env, app_matches.subcommand_matches(&name).unwrap())
        }
        Some(name @ "package") => {
            commands::package::run(&tizen_env, app_matches.subcommand_matches(&name).unwrap())
        }
        Some(name @ "install") => {
            commands::install::run(&tizen_env, app_matches.subcommand_matches(&name).unwrap())
        }
        Some(name @ "run") => {
            commands::run::run(&tizen_env, app_matches.subcommand_matches(&name).unwrap())
        }
        Some(name @ "clean") => {
            commands::clean::run(&tizen_env, app_matches.subcommand_matches(&name).unwrap())
        }
        Some(name @ "dev") => {
            commands::dev::run(&tizen_env, app_matches.subcommand_matches(&name).unwrap())
        }
        _ => Err(TizenError {
            message: "No command matched!".to_string(),
        }),
    };

    match command_result {
        Ok(exit_status) => process::exit(exit_status),
        Err(tizen_error) => {
            eprintln!("{}", &tizen_error.message.bold().red());
            process::exit(1);
        }
    };
}

fn make_tizen_env_args<'a>() -> [Arg<'a, 'a>; 1] {
    [Arg::with_name("emulator")
        .short("e")
        .long("emulator")
        .required(false)
        .takes_value(false)
        .help("Compile to the emulator architecture")]
}

fn make_forward_arg<'a>() -> Arg<'a, 'a> {
    Arg::with_name("forward_args")
        .multiple(true)
        .help("Forward the args")
}

fn make_assume_yes_arg<'a>() -> Arg<'a, 'a> {
    Arg::with_name("assumeyes")
        .short("y")
        .long("assumeyes")
        .multiple(false)
        .required(false)
        .help("Answer yes for all questions")
}

fn make_release_arg<'a>() -> Arg<'a, 'a> {
    Arg::with_name("release")
        .short("r")
        .long("release")
        .multiple(false)
        .required(false)
        .help("Build release")
}

fn get_os_args() -> Vec<String> {
    let mut args: Vec<String> = vec![];
    let mut pos = 0;

    for arg in env::args() {
        if pos == 1 && arg == "tizen" {
            continue;
        }

        args.push(arg);
        pos += 1;
    }

    args
}
