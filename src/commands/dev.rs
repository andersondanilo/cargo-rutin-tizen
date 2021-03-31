use crate::commands;
use crate::error::TizenError;
use crate::TizenEnv;
use clap::ArgMatches;

pub fn run(tizen_env: &TizenEnv, args: &ArgMatches) -> Result<i32, TizenError> {
    commands::build::run(&tizen_env, &args)
        .and_then(|_| commands::package::run(&tizen_env, &args))
        .and_then(|_| commands::install::run(&tizen_env, &args))
        .and_then(|_| commands::run::run(&tizen_env, &args))
}
