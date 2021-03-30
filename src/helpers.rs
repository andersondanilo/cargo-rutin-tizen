use crate::tizen_env::TizenEnv;
use clap::ArgMatches;
use colored::*;
use dialoguer::Confirm;
use std::collections::HashMap;
use std::path::Path;
use std::process::Child;
use std::process::Command;

pub fn run_command(
    tizen_env: &TizenEnv,
    args_m: &ArgMatches,
    name: &str,
    base_args: Vec<&str>,
    last_args: Option<Vec<&str>>,
    include_build_env: bool,
    current_dir: Option<&Path>,
) -> Child {
    let forward_args: Vec<&str> = match args_m.values_of("forward_args") {
        Some(args) => args.collect(),
        None => vec![],
    };

    let last_args = match last_args {
        Some(v) => v,
        None => vec![],
    };

    let working_dir = current_dir.unwrap_or(&tizen_env.base_path);

    println!("Working directory: {}", &working_dir.to_str().unwrap());

    println!(
        "Running: {} {}",
        &name.green().bold(),
        [&base_args[..], &forward_args[..], &last_args[..]]
            .concat()
            .join(" ")
            .green()
            .bold()
    );

    let command = Command::new(name)
        .args([&base_args[..], &forward_args[..], &last_args[..]].concat())
        .envs(if include_build_env {
            make_process_env(tizen_env)
        } else {
            HashMap::new()
        })
        .current_dir(working_dir)
        .spawn();

    match command {
        Ok(c) => c,
        Err(_) => {
            println!("Failed to launch {}", name);
            std::process::exit(1);
        }
    }
}

pub fn make_process_env(tizen_env: &TizenEnv) -> HashMap<String, String> {
    let mut env_map: HashMap<String, String> = tizen_env.get_additional_build_env();

    for config_value in tizen_env.raw_config_values.iter() {
        env_map.insert(config_value.env_key.clone(), config_value.value.clone());
    }

    // for (k, v) in env_map.iter() {
    //     println!("export {}={}", k, v);
    // }

    env_map
}

pub fn ask(question: &str) -> bool {
    Confirm::new()
        .with_prompt(question)
        .interact()
        .unwrap_or(false)
}
