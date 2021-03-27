use clap::Arg;
use clap::{App, AppSettings, SubCommand};
use std::{env, process};
mod commands;
mod error;
mod tizen_env;
use tizen_env::TizenEnv;

fn main() {
    let tizen_env_args = make_tizen_env_args();

    let app_matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("build")
                .about("Wrapper arround cargo build")
                .args(&tizen_env_args),
        )
        .subcommand(
            SubCommand::with_name("package")
                .about("Wrapper arround tizen package")
                .args(&tizen_env_args),
        )
        .subcommand(
            SubCommand::with_name("install")
                .about("Wrapper arround tizen install")
                .args(&tizen_env_args),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Wrapper arround tizen run")
                .args(&tizen_env_args),
        )
        .subcommand(
            SubCommand::with_name("clean")
                .about("Wrapper arround cargo clean and tizen zlean")
                .args(&tizen_env_args),
        )
        .subcommand(
            SubCommand::with_name("config")
                .about("Show config used for building the app")
                .args(&tizen_env_args)
                .arg(
                    Arg::with_name("env_key")
                        .required(false)
                        .takes_value(true)
                        .help("Detail about config key"),
                ),
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

    process::exit(match app_matches.subcommand_name() {
        Some(name @ "config") => {
            commands::config::run(&tizen_env, app_matches.subcommand_matches(&name).unwrap())
        }
        _ => 1,
    });
}

fn make_tizen_env_args() -> [Arg<'static, 'static>; 1] {
    [Arg::with_name("emulator")
        .short("e")
        .long("emulator")
        .required(false)
        .takes_value(false)
        .help("Compile to the emulator architecture")]
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
