use crate::error::TizenError;
use crate::helpers::{ask, run_command};
use crate::tizen_env::TizenEnv;
use clap::ArgMatches;
use colored::*;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub fn run(tizen_env: &TizenEnv, args: &ArgMatches) -> Result<i32, TizenError> {
    let assume_yes = args.is_present("assumeyes");

    let tizen_output_dir = tizen_env.tizen_output_dir();
    let tizen_output_tpk_dir = tizen_env.tizen_output_tpk_dir();

    remove_tizen_output_if_exists(&tizen_output_dir, assume_yes)?;
    create_tizen_output(&tizen_env)?;

    let mut tizen_args = vec![
        "package".to_string(),
        "-t".to_string(),
        "tpk".to_string(),
        "--project".to_string(),
        tizen_output_dir.to_str().unwrap().to_string(),
    ];

    if !tizen_env.security_profile.is_empty() && tizen_env.security_profile != "default" {
        tizen_args.push("--sign".to_string());
        tizen_args.push(tizen_env.security_profile.clone());
    }

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
            message: "cargo tizen package failed!".to_string(),
        });
    }

    Ok(exit_code.code().unwrap())
}

fn remove_tizen_output_if_exists(
    tizen_output_dir: &Path,
    assume_yes: bool,
) -> Result<(), TizenError> {
    if tizen_output_dir.exists() {
        println!(
            "The folder {} already exist",
            &tizen_output_dir.to_str().unwrap().yellow()
        );

        if assume_yes || ask("The folder can be deleted?") {
            fs::remove_dir_all(&tizen_output_dir)?;
            println!("{}", "Folder removed!".green());
            Ok(())
        } else {
            Err(TizenError {
                message: "Operation was cancelled".to_string(),
            })
        }
    } else {
        Ok(())
    }
}

fn create_tizen_output(tizen_env: &TizenEnv) -> Result<(), TizenError> {
    let tizen_output_dir = tizen_env.tizen_output_dir();
    let tizen_output_tpk_dir = tizen_env.tizen_output_tpk_dir();
    let rust_output_dir = tizen_env.rust_output_dir();

    fs::create_dir(&tizen_output_dir)?;

    let mut old_bin = PathBuf::from(&rust_output_dir);
    old_bin.push(&tizen_env.cargo_pkg_name);

    if !old_bin.exists() {
        return Err(TizenError {
            message: format!(
                "Cargo generated bin dont exists: {}",
                &old_bin.to_str().unwrap()
            ),
        });
    }

    for sync_file in tizen_env.sync_files.iter() {
        let mut old_file_path = PathBuf::from(&tizen_env.base_path);
        old_file_path.push(sync_file);

        let mut new_file_path = PathBuf::from(&tizen_output_dir);
        new_file_path.push(&sync_file);

        if old_file_path.exists() {
            if old_file_path.is_file() {
                println!("Sync {}", &sync_file.yellow());
                fs::copy(&old_file_path, &new_file_path)?;
            } else if old_file_path.is_dir() {
                println!("Sync {}", &sync_file.yellow());
                copy_recursive(&old_file_path, &new_file_path)?;
            }
        }

        // @TODO Copy directory
    }

    fs::create_dir(&tizen_output_tpk_dir)?;

    let mut new_bin = tizen_output_tpk_dir.clone();
    new_bin.push(&tizen_env.cargo_pkg_name);

    fs::copy(&old_bin, &new_bin)?;

    if tizen_env.is_release {
        if let Some(strip_bin) = tizen_env.strip_bin() {
            let strip_args = ["--strip-debug", new_bin.to_str().unwrap()];
            println!(
                "Running {} {}",
                strip_bin.green().bold(),
                strip_args.join(" ").green().bold()
            );

            match Command::new(strip_bin).args(&strip_args).spawn() {
                Ok(mut handle) => match handle.wait() {
                    Ok(exit) => {
                        if !exit.success() {
                            println!("{}", "Can't strip bin".bold().red());
                        }
                    }
                    Err(_) => println!("{}", "Can't strip bin".bold().red()),
                },
                Err(_) => println!("{}", "Can't strip bin".bold().red()),
            }
        } else {
            println!("{}", "Strip tool not found!".bold().yellow());
        }
    }

    create_build_info(&tizen_env, &tizen_output_dir, &tizen_output_tpk_dir)?;

    create_project_def(&tizen_env, &tizen_output_dir)?;
    create_project_xml(&tizen_env, &tizen_output_dir)?;

    Ok(())
}

fn create_build_info(
    tizen_env: &TizenEnv,
    tizen_output_dir: &Path,
    tizen_output_bin_dir: &Path,
) -> Result<(), TizenError> {
    let file_name = "build.info";
    let mut build_info_path = PathBuf::from(&tizen_output_bin_dir);
    build_info_path.push(&file_name);

    let mut build_info_file = File::create(build_info_path)?;

    writeln!(
        build_info_file,
        "project-path={}",
        tizen_output_dir.to_str().unwrap()
    )?;
    writeln!(build_info_file, "profile={}", tizen_env.app_profile)?;
    writeln!(build_info_file, "profile-version={}", tizen_env.api_version)?;
    writeln!(build_info_file, "type=app")?;
    writeln!(
        build_info_file,
        "config={}",
        if tizen_env.is_release {
            "Release"
        } else {
            "Debug"
        }
    )?;
    writeln!(build_info_file, "toolchain={}", tizen_env.toolchain)?;
    writeln!(build_info_file, "architecture={}", tizen_env.arch_alias())?;

    println!("Created {}", &file_name.yellow());

    Ok(())
}

fn create_project_def(tizen_env: &TizenEnv, tizen_output_dir: &Path) -> Result<(), TizenError> {
    let file_name = "project_def.prop";
    let mut file_path = PathBuf::from(&tizen_output_dir);
    file_path.push(file_name);

    let mut file = File::create(file_path)?;
    let app_type = match tizen_env.app_ui_type.as_str() {
        "capp" => "app",
        _ => {
            return Err(TizenError {
                message: format!("Unsupported app type {}", &tizen_env.app_ui_type),
            })
        }
    };

    writeln!(file, "APPNAME = {}", &tizen_env.app_label)?;
    writeln!(file, "type = {}", &app_type)?;
    writeln!(
        file,
        "profile = {}-{}",
        &tizen_env.app_profile, &tizen_env.api_version
    )?;
    writeln!(file, "USER_SRCS = ")?;
    writeln!(file, "USER_DEFS = ")?;
    writeln!(file, "USER_INC_DIRS = ")?;
    writeln!(file, "USER_OBJS = ")?;
    writeln!(file, "USER_LIBS = ")?;
    writeln!(file, "USER_EDCS = ")?;

    println!("Created {}", &file_name.yellow());

    Ok(())
}

fn create_project_xml(tizen_env: &TizenEnv, tizen_output_dir: &Path) -> Result<(), TizenError> {
    let file_name = ".project";
    let mut file_path = PathBuf::from(&tizen_output_dir);
    file_path.push(&file_name);

    let mut file = File::create(file_path)?;

    writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(file, "<projectDescription>")?;
    writeln!(file, "<name>{}</name>", &tizen_env.app_label)?;
    writeln!(file, "<comment>{}</comment>", &tizen_env.app_label)?;
    writeln!(file, "<buildSpec></buildSpec>")?;
    writeln!(file, "<natures></natures>")?;
    writeln!(file, "<filteredResources></filteredResources>")?;
    writeln!(file, "</projectDescription>")?;

    println!("Created {}", &file_name.yellow());

    Ok(())
}

pub fn copy_recursive<U: AsRef<Path>, V: AsRef<Path>>(
    from: U,
    to: V,
) -> Result<(), std::io::Error> {
    let mut stack: Vec<PathBuf> = vec![PathBuf::from(from.as_ref())];

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        println!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}
