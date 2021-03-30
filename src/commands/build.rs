use crate::error::TizenError;
use crate::helpers::run_command;
use crate::tizen_env::TizenEnv;
use clap::ArgMatches;

pub fn run(tizen_env: &TizenEnv, args: &ArgMatches) -> Result<i32, TizenError> {
    // @TODO: Pass release arg

    let mut handle = run_command(
        &tizen_env,
        &args,
        "cargo",
        vec!["build", &format!("--target={}", &tizen_env.rust_triple)],
        None,
        true,
        None,
    );

    let exit_code = handle.wait().expect("Failed to wait on child");

    if !exit_code.success() {
        eprintln!("cargo tizen build failed!");
    }

    Ok(exit_code.code().unwrap())
}
