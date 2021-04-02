use crate::error::TizenError;
use crate::helpers::run_command;
use crate::tizen_env::TizenEnv;
use clap::ArgMatches;

pub fn run(tizen_env: &TizenEnv, args: &ArgMatches) -> Result<i32, TizenError> {
    let tizen_args = vec![
        "run".to_string(),
        "-p".to_string(),
        tizen_env.app_id.clone(),
    ];

    let mut handle = run_command(
        &tizen_env,
        &args,
        &tizen_env.tizen_bin,
        &tizen_args,
        None,
        false,
        Some(&tizen_env.base_path),
    );

    let exit_code = handle.wait().expect("Failed to wait on child");

    if !exit_code.success() {
        return Err(TizenError {
            message: "cargo tizen run failed!".to_string(),
        });
    }

    Ok(exit_code.code().unwrap())
}
