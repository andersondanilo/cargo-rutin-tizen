use crate::error::TizenError;
use clap::ArgMatches;
use std::collections::HashMap;
use std::fs;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use sxd_document::{parser, Package};
use sxd_xpath::Context;
use sxd_xpath::Factory;
use toml::Value;

pub struct TizenEnv {
    pub raw_config_values: Vec<ConfigValue>,

    pub base_path: PathBuf,
    pub studio_path: PathBuf,
    pub tizen_bin: String,
    pub is_emulator: bool,
    pub api_version: String,
    pub app_profile: String,
    pub rootstrap_path: PathBuf,
    pub tizen_triple: String,
    pub rust_triple: String,
    pub toolchain: String,
    pub toolchain_path: PathBuf,
    pub rust_linker: String,
    pub app_id: String,
    pub app_version: String,
    pub app_package: String,
    pub app_exec: String,
    pub app_label: String,
    pub app_ui_type: String,
    pub cargo_pkg_name: String,
    pub sync_files: Vec<String>,
}

impl TizenEnv {
    pub fn from_cargo_config(cwd: &Path, app_m: &ArgMatches) -> Result<Self, TizenError> {
        let config_provider = ConfigProvider::new(cwd.to_path_buf(), &app_m)?;

        let studio_path = config_provider.get_value(&ConfigType::StudioPath)?;
        let is_emulator = config_provider.get_value(&ConfigType::IsEmulator)?;
        let api_version = config_provider.get_value(&ConfigType::ApiVersion)?;
        let app_profile = config_provider.get_value(&ConfigType::AppProfile)?;
        let rootstrap_path = config_provider.get_value(&ConfigType::RootstrapPath)?;
        let device_triple = config_provider.get_value(&ConfigType::DeviceTriple)?;
        let emulator_triple = config_provider.get_value(&ConfigType::EmulatorTriple)?;
        let tizen_triple = config_provider.get_value(&ConfigType::SelectedTriple)?;
        let toolchain = config_provider.get_value(&ConfigType::Toolchain)?;
        let rust_triple = config_provider.get_value(&ConfigType::RustTriple)?;
        let toolchain_path = config_provider.get_value(&ConfigType::ToolchainPath)?;
        let rust_linker = config_provider.get_value(&ConfigType::RustLinker)?;
        let app_id = config_provider.get_value(&ConfigType::AppId)?;
        let app_version = config_provider.get_value(&ConfigType::AppVersion)?;
        let app_package = config_provider.get_value(&ConfigType::AppPackage)?;
        let app_exec = config_provider.get_value(&ConfigType::AppExec)?;
        let tizen_bin = config_provider.get_value(&ConfigType::TizenBin)?;
        let app_label = config_provider.get_value(&ConfigType::AppLabel)?;
        let app_ui_type = config_provider.get_value(&ConfigType::AppUiType)?;
        let sync_files = config_provider.get_value(&ConfigType::SyncFiles)?;

        let cargo_pkg_name = match config_provider.get_cargo_value("package.name") {
            Some(s) => s,
            None => {
                return Err(TizenError {
                    message: "Can't get package.name from Cargo.toml".to_string(),
                })
            }
        };

        let sync_files_array: Vec<&str> = sync_files.value.split(',').collect();
        let sync_files_array: Vec<String> =
            sync_files_array.iter().map(|s| s.to_string()).collect();

        Ok(Self {
            base_path: PathBuf::from(cwd),
            studio_path: PathBuf::from(&studio_path.value),
            is_emulator: str_to_bool(&studio_path.value),
            api_version: api_version.value.clone(),
            app_profile: app_profile.value.clone(),
            rootstrap_path: PathBuf::from(&rootstrap_path.value),
            tizen_triple: tizen_triple.value.clone(),
            rust_triple: rust_triple.value.clone(),
            toolchain: toolchain.value.clone(),
            toolchain_path: PathBuf::from(&toolchain_path.value),
            rust_linker: rust_linker.value.clone(),
            app_id: app_id.value.clone(),
            app_version: app_version.value.clone(),
            app_package: app_package.value.clone(),
            app_exec: app_exec.value.clone(),
            tizen_bin: tizen_bin.value.clone(),
            cargo_pkg_name,
            sync_files: sync_files_array,
            app_label: app_label.value.clone(),
            app_ui_type: app_ui_type.value.clone(),
            raw_config_values: vec![
                studio_path,
                is_emulator,
                api_version,
                app_profile,
                rootstrap_path,
                tizen_triple,
                device_triple,
                emulator_triple,
                toolchain,
                rust_triple,
                toolchain_path,
                rust_linker,
                app_id,
                app_version,
                app_package,
                app_exec,
                tizen_bin,
                app_label,
                sync_files,
                app_ui_type,
            ],
        })
    }

    pub fn get_additional_build_env(&self) -> HashMap<String, String> {
        let mut envs: HashMap<String, String> = HashMap::new();

        let rootstrap_path = match self.rootstrap_path.to_str() {
            Some(s) => String::from(s),
            None => "".to_string(),
        };

        envs.insert("PKG_CONFIG_SYSROOT_DIR".to_string(), rootstrap_path.clone());
        envs.insert(
            "PKG_CONFIG_LIBDIR".to_string(),
            format!("{}/usr/lib/pkgconfig", &rootstrap_path),
        );
        envs.insert("PKG_CONFIG_PATH".to_string(), "".to_string());
        envs.insert("PKG_CONFIG_ALLOW_CROSS".to_string(), "1".to_string());
        envs.insert(
            "RUSTFLAGS".to_string(),
            format!("-C link-args=--sysroot={}", &rootstrap_path),
        );

        envs.insert(
            format!(
                "CARGO_TARGET_{}_LINKER",
                &self.rust_triple.clone().to_uppercase().replace('-', "_")
            ),
            self.rust_linker.clone(),
        );

        envs
    }

    pub fn rust_output_dir(&self, is_release: bool) -> PathBuf {
        let mut out_path = self.base_path.clone();
        out_path.push("target");
        out_path.push(&self.rust_triple);
        out_path.push(if is_release { "release" } else { "debug" });

        out_path
    }

    pub fn tizen_output_dir(&self, is_release: bool) -> PathBuf {
        let mut out_path = self.rust_output_dir(is_release);

        out_path.push("tizen-tpk");

        out_path
        // let mut out_path = self.base_path.clone();
        // out_path.push(if is_release { "Release" } else { "DebugTest" });

        // out_path
    }

    pub fn arch_alias(&self) -> String {
        if self.tizen_triple.contains("arm") {
            "arm".to_string()
        } else {
            "x86".to_string()
        }
    }
}

fn str_to_bool(val: &str) -> bool {
    val == "1" || val == "true"
}

#[derive(Copy, Clone)]
pub enum ConfigType {
    StudioPath,
    AppProfile,
    AppId,
    AppVersion,
    AppPackage,
    ApiVersion,
    IsEmulator,
    AppExec,
    RootstrapPath,
    DeviceTriple,
    EmulatorTriple,
    SelectedTriple,
    RustTriple,
    Toolchain,
    ToolchainPath,
    RustLinker,
    TizenBin,
    AppLabel,
    AppUiType,
    SyncFiles,
}

pub enum ConfigFrom {
    Env,
    Cargo,
    Manifest,
    Arg,
    Default,
}

pub struct ConfigValue {
    pub config_type: ConfigType,
    pub from: ConfigFrom,
    pub value: String,
    pub env_key: String,
    pub cargo_key: Option<String>,
    pub manifest_key: Option<String>,
}

struct ConfigProvider<'a> {
    arg_matches: &'a ArgMatches<'a>,
    cargo_files: Vec<Value>,
    cargo_build_file: Value,
    cargo_default_file: Value,
    manifest_document: Package,
}

impl<'a> ConfigProvider<'a> {
    fn new(base_path: PathBuf, arg_matches: &'a ArgMatches<'a>) -> Result<Self, TizenError> {
        let mut manifest_path = base_path.clone();
        manifest_path.push("tizen-manifest.xml");

        let cargo_files = Self::get_cargo_config_files(&base_path);
        let cargo_build_file = Self::get_cargo_build_file(&base_path)?;
        let cargo_default_file = Self::get_cargo_default_file();

        Ok(Self {
            arg_matches,
            cargo_files,
            cargo_build_file,
            cargo_default_file,
            manifest_document: parser::parse(&std::fs::read_to_string(manifest_path.as_path())?)?,
        })
    }

    fn get_value(&self, config_type: &ConfigType) -> Result<ConfigValue, TizenError> {
        let dynamic_key: Option<String> = match config_type {
            ConfigType::RustTriple => match self.get_value(&ConfigType::SelectedTriple) {
                Ok(selected_triple) => Some(format!(
                    "tizen.target.{}.rust_triple",
                    &selected_triple.value
                )),
                Err(_) => None,
            },
            ConfigType::ToolchainPath => match self.get_value(&ConfigType::SelectedTriple) {
                Ok(selected_triple) => Some(format!(
                    "tizen.target.{}.toolchain_path",
                    &selected_triple.value
                )),
                Err(_) => None,
            },
            ConfigType::RustLinker => match self.get_value(&ConfigType::SelectedTriple) {
                Ok(selected_triple) => Some(format!(
                    "tizen.target.{}.rust_linker",
                    &selected_triple.value
                )),
                Err(_) => None,
            },
            _ => None,
        };

        self.get_custom_value(&config_type, dynamic_key)
    }

    fn get_custom_value(
        &self,
        config_type: &ConfigType,
        cargo_key: Option<String>,
    ) -> Result<ConfigValue, TizenError> {
        let cargo_key = match cargo_key {
            Some(_) => cargo_key,
            None => Self::get_cargo_key(config_type),
        };

        let env_key = match Self::get_env_key(&config_type, cargo_key.clone()) {
            Some(env_key) => env_key,
            None => {
                return Err(TizenError {
                    message: "Invalid config type or name".to_string(),
                })
            }
        };

        let manifest_key = Self::get_manifest_key(&config_type);

        let base_config_value = ConfigValue {
            config_type: *config_type,
            from: ConfigFrom::Env,
            value: "".to_string(),
            env_key: env_key.clone(),
            cargo_key: cargo_key.clone(),
            manifest_key: manifest_key.clone(),
        };

        if let Ok(str_value) = std::env::var(&env_key) {
            return Ok(ConfigValue {
                from: ConfigFrom::Env,
                value: str_value,
                ..base_config_value
            });
        }

        if let Some(manifest_key) = &manifest_key {
            if let Some(str_value) = self.get_manifest_value(&manifest_key) {
                return Ok(ConfigValue {
                    from: ConfigFrom::Manifest,
                    value: str_value,
                    ..base_config_value
                });
            }
        }

        if let Some(cargo_key) = &cargo_key {
            if let Some(str_value) = self.get_cargo_value(&cargo_key) {
                return Ok(ConfigValue {
                    from: ConfigFrom::Cargo,
                    value: str_value,
                    ..base_config_value
                });
            }
        }

        if let Some(str_value) = self.get_arg_value(&config_type) {
            return Ok(ConfigValue {
                from: ConfigFrom::Arg,
                value: str_value,
                ..base_config_value
            });
        }

        if let Some(str_value) = self.get_default_value(&config_type, &cargo_key) {
            return Ok(ConfigValue {
                from: ConfigFrom::Default,
                value: str_value,
                ..base_config_value
            });
        }

        let mut error_list: Vec<String> = vec![];

        if let Some(manifest_key) = &manifest_key {
            error_list.push(format!(
                "Config '{}' not found in manifest xml",
                &manifest_key
            ));
        }

        if let Some(cargo_key) = &cargo_key {
            error_list.push(format!("Config '{}' not found in cargo config", &cargo_key));
        }

        error_list.push(format!("Config '{}' not found in env", &env_key));

        Err(TizenError {
            message: error_list.join("\n"),
        })
    }

    fn get_manifest_value(&self, path: &str) -> Option<String> {
        let document = self.manifest_document.as_document();

        let expression = match Factory::new().build(path) {
            Ok(e) => match e {
                Some(e) => e,
                None => return None,
            },
            Err(_) => return None,
        };

        let mut context = Context::new();
        context.set_namespace("ns", "http://tizen.org/ns/packages");

        match expression.evaluate(&context, document.root()) {
            Ok(value) => match value.string() {
                str_value if !str_value.is_empty() => Some(str_value),
                _ => None,
            },
            Err(_) => None,
        }
    }

    fn get_cargo_value(&self, key: &str) -> Option<String> {
        if let Some(result_str) = Self::get_toml_str(&self.cargo_build_file, &key) {
            return Some(result_str);
        }

        if !key.starts_with("package.") {
            for cargo_file in self.cargo_files.iter() {
                if let Some(result_str) = Self::get_toml_str(&cargo_file, &key) {
                    return Some(result_str);
                }
            }
        }

        None
    }

    fn get_arg_value(&self, config_type: &ConfigType) -> Option<String> {
        match config_type {
            ConfigType::IsEmulator => match self.arg_matches.is_present("emulator") {
                true => Some("true".to_string()),
                _ => None,
            },
            _ => None,
        }
    }

    fn get_default_value(
        &self,
        config_type: &ConfigType,
        cargo_key: &Option<String>,
    ) -> Option<String> {
        match cargo_key {
            Some(str_val) => {
                if let Some(result_str) = Self::get_toml_str(&self.cargo_default_file, &str_val) {
                    return Some(result_str);
                }
            }
            None => (),
        }

        self.get_default_computed_value(&config_type).ok()
    }

    fn get_default_computed_value(&self, config_type: &ConfigType) -> Result<String, TizenError> {
        match config_type {
            ConfigType::RootstrapPath => {
                let api_version = self.get_value(&ConfigType::ApiVersion)?.value;
                let app_profile = self.get_value(&ConfigType::AppProfile)?.value;
                let is_emulator = str_to_bool(&self.get_value(&ConfigType::IsEmulator)?.value);

                let mut path = PathBuf::from(&self.get_value(&ConfigType::StudioPath)?.value);
                path.push("platforms");
                path.push(format!("tizen-{}", api_version));
                path.push(format!("{}", app_profile));
                path.push("rootstraps");
                path.push(format!(
                    "{}-{}-{}.core",
                    app_profile,
                    api_version,
                    if is_emulator { "emulator" } else { "device" }
                ));

                match path.to_str() {
                    Some(str_value) => Ok(str_value.to_string()),
                    None => Err(TizenError {
                        message: "Can't get path".to_string(),
                    }),
                }
            }
            ConfigType::SelectedTriple => {
                let is_emulator = str_to_bool(&self.get_value(&ConfigType::IsEmulator)?.value);
                if is_emulator {
                    Ok(self.get_value(&ConfigType::EmulatorTriple)?.value)
                } else {
                    Ok(self.get_value(&ConfigType::DeviceTriple)?.value)
                }
            }
            ConfigType::Toolchain => {
                let tizen_studio_path = self.get_value(&ConfigType::StudioPath)?.value;
                let selected_triple = self.get_value(&ConfigType::SelectedTriple)?.value;

                let mut path = PathBuf::from(tizen_studio_path);
                path.push("tools");

                if !path.exists() || !path.is_dir() {
                    return Err(TizenError {
                        message: "Studio tools path does not exists!".to_string(),
                    });
                }

                let mut available_toolchains: Vec<String> = fs::read_dir(path)?
                    .filter_map(|entry_result| entry_result.ok())
                    .filter_map(|entry| entry.file_name().into_string().ok())
                    .filter(|str_value| str_value.starts_with(&selected_triple))
                    .filter(|str_value| str_value.contains("gcc"))
                    .map(|str_value| str_value.replace(&format!("{}-", &selected_triple), ""))
                    .collect();

                available_toolchains.sort();
                available_toolchains.reverse();

                match available_toolchains.first() {
                    Some(str_value) => Ok(str_value.to_owned()),
                    None => Err(TizenError {
                        message: "No toolchain found".to_string(),
                    }),
                }
            }
            ConfigType::ToolchainPath => {
                let tizen_studio_path = self.get_value(&ConfigType::StudioPath)?.value;
                let tizen_toolchain = self.get_value(&ConfigType::Toolchain)?.value;
                let selected_triple = self.get_value(&ConfigType::SelectedTriple)?.value;

                let mut path = PathBuf::from(tizen_studio_path);
                path.push("tools");
                path.push(format!("{}-{}", selected_triple, tizen_toolchain));
                path.push("bin");

                match path.to_str() {
                    Some(str_value) => Ok(str_value.to_string()),
                    None => Err(TizenError {
                        message: "Can't get path".to_string(),
                    }),
                }
            }
            ConfigType::TizenBin => {
                let tizen_studio_path = self.get_value(&ConfigType::StudioPath)?.value;

                let mut path = PathBuf::from(tizen_studio_path);
                path.push("tools");
                path.push("ide");
                path.push("bin");
                path.push("tizen");

                match path.to_str() {
                    Some(str_value) => Ok(str_value.to_string()),
                    None => Err(TizenError {
                        message: "Can't get path".to_string(),
                    }),
                }
            }
            ConfigType::RustLinker => {
                let toolchain_path = self.get_value(&ConfigType::ToolchainPath)?.value;
                let selected_triple = self.get_value(&ConfigType::SelectedTriple)?.value;

                let mut path = PathBuf::from(&toolchain_path);
                path.push(&toolchain_path);
                path.push(format!("{}-gcc", &selected_triple));

                match path.to_str() {
                    Some(str_value) => Ok(str_value.to_string()),
                    None => Err(TizenError {
                        message: "Can't get path".to_string(),
                    }),
                }
            }
            _ => Err(TizenError {
                message: "no computed for value".to_string(),
            }),
        }
    }

    fn get_cargo_key(config_type: &ConfigType) -> Option<String> {
        match config_type {
            ConfigType::StudioPath => Some("tizen.studio_path".to_string()),
            ConfigType::AppProfile => Some("tizen.app_profile".to_string()),
            ConfigType::AppId => None,
            ConfigType::AppVersion => None,
            ConfigType::AppPackage => None,
            ConfigType::ApiVersion => Some("tizen.api_version".to_string()),
            ConfigType::IsEmulator => Some("tizen.is_emulator".to_string()),
            ConfigType::AppExec => None,
            ConfigType::RootstrapPath => Some("tizen.rootstrap_path".to_string()),
            ConfigType::DeviceTriple => Some("tizen.device_triple".to_string()),
            ConfigType::EmulatorTriple => Some("tizen.emulator_triple".to_string()),
            ConfigType::SelectedTriple => Some("tizen.selected_triple".to_string()),
            ConfigType::Toolchain => Some("tizen.toolchain".to_string()),
            ConfigType::TizenBin => Some("tizen.bin_path".to_string()),
            ConfigType::SyncFiles => Some("tizen.sync_files".to_string()),
            ConfigType::AppLabel => Some("tizen.app_label".to_string()),
            ConfigType::AppUiType => Some("tizen.app_ui_type".to_string()),
            _ => None,
        }
    }

    fn get_env_key(config_type: &ConfigType, cargo_key: Option<String>) -> Option<String> {
        let base_name = match cargo_key {
            Some(_) => cargo_key,
            None => match config_type {
                ConfigType::AppId => Some("TIZEN_APP_ID".to_string()),
                ConfigType::AppVersion => Some("TIZEN_APP_VERSION".to_string()),
                ConfigType::AppPackage => Some("TIZEN_APP_PACKAGE".to_string()),
                ConfigType::AppExec => Some("TIZEN_APP_EXEC".to_string()),
                _ => None,
            },
        };

        base_name.map(|str_value| str_value.to_uppercase().replace(".", "_").replace("-", "_"))
    }

    fn get_manifest_key(config_type: &ConfigType) -> Option<String> {
        match config_type {
            ConfigType::AppId => Some("/ns:manifest/ns:ui-application/@appid".to_string()),
            ConfigType::AppVersion => Some("/ns:manifest/@version".to_string()),
            ConfigType::ApiVersion => Some("/ns:manifest/@api-version".to_string()),
            ConfigType::AppPackage => Some("/ns:manifest/@package".to_string()),
            ConfigType::AppExec => Some("/ns:manifest/ns:ui-application/@exec".to_string()),
            ConfigType::AppLabel => Some("/ns:manifest/ns:ui-application/ns:label".to_string()),
            ConfigType::AppProfile => Some("/ns:manifest/ns:profile/@name".to_string()),
            ConfigType::AppUiType => Some("/ns:manifest/ns:ui-application/@type".to_string()),
            _ => None,
        }
    }

    fn get_cargo_build_file(base_path: &Path) -> Result<Value, TizenError> {
        let mut cargo_build_path = PathBuf::from(base_path);
        cargo_build_path.push("Cargo.toml");

        match cargo_build_path.exists() {
            true => match read_to_string(&cargo_build_path) {
                Ok(content) => match content.parse::<Value>() {
                    Ok(toml_value) => Ok(toml_value),
                    Err(_) => Err(TizenError {
                        message: format!(
                            "Can't parse {}",
                            &cargo_build_path.to_str().unwrap_or("")
                        ),
                    }),
                },
                Err(_) => Err(TizenError {
                    message: format!("Can't read {}", &cargo_build_path.to_str().unwrap_or("")),
                }),
            },
            false => Err(TizenError {
                message: format!(
                    "File does not exists {}",
                    &cargo_build_path.to_str().unwrap_or("")
                ),
            }),
        }
    }

    fn get_cargo_default_file() -> Value {
        let cargo_default_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "res/default-cargo.toml"
        ));
        cargo_default_str.parse::<Value>().unwrap()
    }

    fn get_cargo_config_files(base_path: &Path) -> Vec<Value> {
        let mut config_path = PathBuf::from(base_path);

        config_path.push(".cargo");
        config_path.push("config");
        config_path.set_extension("toml");

        let mut first_vector: Vec<Value> = match config_path.exists() {
            true => match read_to_string(&config_path) {
                Ok(content) => match content.parse::<Value>() {
                    Ok(toml_value) => vec![toml_value],
                    Err(_) => vec![],
                },
                Err(_) => vec![],
            },
            false => vec![],
        };

        let mut new_path = PathBuf::from(base_path);

        let last_vector: Vec<Value> = match new_path.pop() {
            true => match Self::home_dir() {
                Some(home_path) => {
                    if home_path == new_path {
                        vec![]
                    } else {
                        Self::get_cargo_config_files(&new_path)
                    }
                }
                None => vec![],
            },
            false => vec![],
        };

        first_vector.extend(last_vector);
        first_vector
    }

    fn home_dir() -> Option<PathBuf> {
        match std::env::var("HOME") {
            Ok(str_value) => Some(PathBuf::from(str_value)),
            Err(_) => None,
        }
    }

    fn get_toml_str(toml_value: &Value, key: &str) -> Option<String> {
        let result_opt =
            key.split('.').fold(
                Some(toml_value),
                |old_value_opt, piece| match old_value_opt {
                    Some(old_value) => old_value.get(piece),
                    _ => None,
                },
            );

        match result_opt {
            Some(val) => Self::toml_value_2_str(&val),
            None => None,
        }
    }

    fn toml_value_2_str(val: &Value) -> Option<String> {
        match val {
            Value::String(val) => Some(val.to_string()),
            Value::Integer(val) => Some(val.to_string()),
            Value::Boolean(val) => Some(val.to_string()),
            Value::Float(val) => Some(val.to_string()),
            Value::Array(val) => Some(
                val.iter()
                    .filter_map(|v| Self::toml_value_2_str(v))
                    .collect::<Vec<String>>()
                    .join(","),
            ),
            _ => None,
        }
    }
}
