use crate::error::TizenError;
use crate::helpers::run_command;
use crate::tizen_env::TizenEnv;
use clap::ArgMatches;

pub fn run(tizen_env: &TizenEnv, args: &ArgMatches) -> Result<i32, TizenError> {
    let mut cargo_args: Vec<String> = vec![
        "build".to_string(),
        format!("--target={}", &tizen_env.rust_triple),
    ];

    if tizen_env.is_release {
        cargo_args.push("--release".to_string());
    }

    let mut handle = run_command(&tizen_env, &args, "cargo", &cargo_args, None, true, None);

    let exit_code = handle.wait().expect("Failed to wait on child");

    if !exit_code.success() {
        return Err(TizenError {
            message: "cargo tizen build failed!".to_string(),
        });
    }

    Ok(exit_code.code().unwrap())
}
