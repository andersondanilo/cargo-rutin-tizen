use crate::error::TizenError;
use crate::tizen_env::{ConfigFrom, TizenEnv};
use clap::ArgMatches;
use cli_table::{print_stdout, Cell, Table};
use colored::*;

pub fn run(tizen_env: &TizenEnv, args: &ArgMatches) -> Result<i32, TizenError> {
    match args.value_of("env_key") {
        Some(str_value) => show_detail(tizen_env, String::from(str_value)),
        None => list_configs(&tizen_env),
    }
}

fn show_detail(tizen_env: &TizenEnv, env_key: String) -> Result<i32, TizenError> {
    let config_value = tizen_env
        .raw_config_values
        .iter()
        .find(|v| v.env_key == env_key);

    match config_value {
        Some(config_value) => {
            let table = vec![
                vec!["env key".cell(), config_value.env_key.clone().cell()],
                vec!["value".cell(), config_value.value.clone().cell()],
                vec!["from".cell(), from_to_s(&config_value.from).cell()],
                vec![
                    "cargo key".cell(),
                    config_value
                        .cargo_key
                        .clone()
                        .unwrap_or_else(|| "".to_string())
                        .cell(),
                ],
                vec![
                    "manifest key".cell(),
                    config_value
                        .manifest_key
                        .clone()
                        .unwrap_or_else(|| "".to_string())
                        .cell(),
                ],
            ]
            .table();

            assert!(print_stdout(table).is_ok());

            Ok(0)
        }
        None => Err(TizenError {
            message: format!("No config named {}", env_key),
        }),
    }
}

fn list_configs(tizen_env: &TizenEnv) -> Result<i32, TizenError> {
    println!("{}", "Configurable values:".green().bold());

    for raw_value in tizen_env.raw_config_values.iter() {
        println!("{}={}", raw_value.env_key, raw_value.value);
    }

    println!(
        "{} {} {}",
        "Run".green(),
        "cargo tizen config NAME_OF_CONFIG".yellow().bold(),
        "to see more info".green()
    );

    println!("\n{}", "Other env variables:".green().bold());

    for (key, value) in tizen_env.get_additional_build_env() {
        println!("{}={}", &key, &value);
    }

    Ok(0)
}

fn from_to_s(config_from: &ConfigFrom) -> String {
    match config_from {
        ConfigFrom::Env => "env".to_string(),
        ConfigFrom::Arg => "cli args".to_string(),
        ConfigFrom::Cargo => "cargo file".to_string(),
        ConfigFrom::Manifest => "manifest".to_string(),
        ConfigFrom::Default => "default".to_string(),
    }
}
