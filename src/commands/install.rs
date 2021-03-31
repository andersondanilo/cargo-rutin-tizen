use crate::error::TizenError;
use crate::helpers::run_command;
use crate::tizen_env::TizenEnv;
use clap::ArgMatches;

pub fn run(tizen_env: &TizenEnv, args: &ArgMatches) -> Result<i32, TizenError> {
    let tizen_output_tpk_dir = tizen_env.tizen_output_tpk_dir();

    let tizen_args = vec![
        "install".to_string(),
        "-n".to_string(),
        tizen_env.tpk_name(),
    ];

    let mut handle = run_command(
        &tizen_env,
        &args,
        &tizen_env.tizen_bin,
        &tizen_args,
        None,
        false,
        Some(&tizen_output_tpk_dir),
    );

    let exit_code = handle.wait().expect("Failed to wait on child");

    if !exit_code.success() {
        return Err(TizenError {
            message: "cargo tizen install failed!".to_string(),
        });
    }

    Ok(exit_code.code().unwrap())
}
