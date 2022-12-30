use super::specification::ScenarioConfig;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Specify the scenario to run.
    #[command(arg_required_else_help = true)]
    Scenario(Scenario),
}

/// Scenario subcommand
#[derive(Debug, Args)]
struct Scenario {
    /// Scenario to load, can be one of the builtins, or a yaml file specifying the scenario. Use
    /// 'list' as a special scenario to list the builtins.
    #[arg(value_hint = clap::ValueHint::FilePath)]
    scenario: String,

    #[cfg(feature = "unit_control_wasm")]
    #[arg(short, long, verbatim_doc_comment)]
    /// Direct override of the path attribute of wasm controllers. Use with --wasm team_a:path_to_module.wasm --wasm team_b:path_to_module.wasm.
    wasm: Vec<String>,

    /// Override properties from the configuration, the format of these keys is a bit bespoke and verbose.
    /// In general, it is prefered to use the --wasm argument.
    /// It is limited to (depending on enabled controllers)
    /// - "control:controller_name:wasm:path:foo.wasm" -> Set controller by 'controller_name' to wasm and use 'foo.wasm'.
    /// - "control:controller_name:wasm:fuel_per_update:1000"  (or none) -> Set fuel_per_update to none or value.
    /// - "control:controller_name:wasm:fuel_for_setup:1000"  (or none) -> Set fuel_for_setup to none or value.
    /// - "control:controller_name:none" -> Set the controller with 'controller_name' to none.
    #[arg(short, long, verbatim_doc_comment)]
    config: Vec<String>,
}

pub fn parse_args() -> Result<ScenarioConfig, Box<dyn std::error::Error>> {
    let args = Cli::parse();
    match args.command {
        Commands::Scenario(scenario) => {
            if scenario.scenario == "list" {
                let available = super::reader::builtin_scenarios();
                println!("Available scenarios:");
                for name in available {
                    println!("  {}", name);
                }
                std::process::exit(0);
            }

            // Check if it is a built in scenario
            let specification =
                if super::reader::builtin_scenarios().contains(&scenario.scenario.as_str()) {
                    super::reader::get_builtin_scenario(&scenario.scenario)?
                } else {
                    // It wasn't... well, lets hope that it is a file...
                    let p = std::path::PathBuf::from(&scenario.scenario);
                    super::reader::read_scenario_config(&p)?
                };

            #[cfg(not(feature = "unit_control_wasm"))]
            let extra_config: Vec<String> = vec![];

            #[cfg(feature = "unit_control_wasm")]
            let extra_config = {
                let mut extra_config = vec![];
                for wasm in scenario.wasm.iter() {
                    let split = wasm
                        .split(":")
                        .map(|v| v.to_owned())
                        .collect::<Vec<String>>();
                    if split.len() != 2 {
                        return Err(Box::<dyn std::error::Error>::from(
                            "expected ':' between team name and path",
                        ));
                    }
                    extra_config.push(format!("control:{}:wasm:path:{}", split[0], split[1]));
                }
                extra_config
            };

            // Apply any config overrides...
            let config_strs: Vec<&str> = scenario
                .config
                .iter()
                .chain(extra_config.iter())
                .map(|v| v.as_str())
                .collect();
            apply_config(&config_strs, specification)
        }
    }
}

// Well, this function is a bit... much.
fn apply_config(
    config: &[&str],
    scenario: ScenarioConfig,
) -> Result<ScenarioConfig, Box<dyn std::error::Error>> {
    let mut scenario = scenario;
    let make_error = |s: &str| Box::<dyn std::error::Error>::from(s);
    use crate::config::specification::ControllerType;

    for c in config.iter() {
        let mut tokens = c.split(":");

        match tokens.next().ok_or_else(|| make_error("expected token"))? {
            "control" => {
                let section = &mut scenario.spawn_config.control_config;
                let key = tokens
                    .next()
                    .ok_or_else(|| make_error("expected control key"))?;
                let control = section
                    .get_mut(key)
                    .ok_or_else(|| make_error(format!("key {} didnt exist", key).as_str()))?;
                let control_type = tokens
                    .next()
                    .ok_or_else(|| make_error("expected control type"))?;
                match control_type {
                    #[cfg(feature = "unit_control_wasm")]
                    "wasm" => {
                        let wasm = if let ControllerType::Wasm(ref mut v) = control {
                            v
                        } else {
                            *control = ControllerType::Wasm(Default::default());
                            if let ControllerType::Wasm(ref mut v) = control {
                                v
                            } else {
                                unreachable!()
                            }
                        };
                        let field_key = tokens
                            .next()
                            .ok_or_else(|| make_error("expected wasm control field"))?;
                        match field_key {
                            "path" => {
                                wasm.path = tokens
                                    .next()
                                    .ok_or_else(|| make_error("expected path to file"))?
                                    .to_owned();
                            }
                            "fuel_per_update" => {
                                let value_str = tokens.next().ok_or_else(|| {
                                    make_error("expected value for fuel_per_update")
                                })?;
                                let value = if value_str.to_lowercase() == "none" {
                                    None
                                } else {
                                    Some(value_str.parse::<u64>()?)
                                };
                                wasm.fuel_per_update = value;
                            }
                            "fuel_for_setup" => {
                                let value_str = tokens.next().ok_or_else(|| {
                                    make_error("expected value for fuel_for_setup")
                                })?;
                                let value = if value_str.to_lowercase() == "none" {
                                    None
                                } else {
                                    Some(value_str.parse::<u64>()?)
                                };
                                wasm.fuel_for_setup = value;
                            }
                            _ => {
                                return Err(make_error(
                                    format!("{} is unhandled for wasm", field_key).as_str(),
                                ));
                            }
                        }
                    }
                    "none" => {
                        *control = ControllerType::None;
                    }
                    _ => {
                        return Err(make_error(
                            format!("cannot handle control {} overrides", control_type).as_str(),
                        ));
                    }
                }
            }
            _ => {
                return Err(make_error("expected valid token; 'control'"));
            }
        }
    }
    Ok(scenario)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_config_cli_control_config_none() {
        use crate::config::specification::ControllerType;
        use crate::config::specification::SpawnConfig;
        let mut control_config = std::collections::HashMap::<String, ControllerType>::new();
        control_config.insert("a".to_owned(), ControllerType::InterfacePrinter);
        control_config.insert("b".to_owned(), ControllerType::None);
        let v = ScenarioConfig {
            spawn_config: SpawnConfig {
                control_config,
                ..Default::default()
            },
            ..Default::default()
        };
        let config: Vec<String> = vec!["control:a:none".to_owned()];
        let strslice: Vec<&str> = config.iter().map(|v| v.as_str()).collect();
        let r = apply_config(&strslice, v);
        println!("r; {r:?}");
        assert!(r.is_ok());
        let r = r.unwrap();
        let a = r
            .spawn_config
            .control_config
            .get("a")
            .expect("a should still exist");
        assert_eq!(*a, ControllerType::None);
    }

    #[cfg(feature = "unit_control_wasm")]
    #[test]
    fn test_config_cli_control_config() {
        use crate::config::specification::ControllerType;
        use crate::config::specification::SpawnConfig;
        let mut control_config = std::collections::HashMap::<String, ControllerType>::new();
        control_config.insert("a".to_owned(), ControllerType::None);
        control_config.insert("b".to_owned(), ControllerType::None);
        let v = ScenarioConfig {
            spawn_config: SpawnConfig {
                control_config,
                ..Default::default()
            },
            ..Default::default()
        };
        let config: Vec<String> = vec![
            "control:a:wasm:path:foo.wasm".to_owned(),
            "control:a:wasm:fuel_per_update:42".to_owned(),
            "control:a:wasm:fuel_for_setup:1337".to_owned(),
        ];
        let strslice: Vec<&str> = config.iter().map(|v| v.as_str()).collect();
        let r = apply_config(&strslice, v);
        println!("r; {r:?}");
        assert!(r.is_ok());
        let r = r.unwrap();
        let a = r
            .spawn_config
            .control_config
            .get("a")
            .expect("a should still exist");
        if let ControllerType::Wasm(wasm) = a {
            assert_eq!(wasm.path, "foo.wasm");
            assert_eq!(wasm.fuel_per_update, Some(42));
            assert_eq!(wasm.fuel_for_setup, Some(1337));
        } else {
            panic!("not of expected wasm type");
        };

        let config_override: Vec<String> = vec![
            "control:a:wasm:fuel_per_update:none".to_owned(),
            "control:a:wasm:fuel_for_setup:none".to_owned(),
        ];
        let strslice: Vec<&str> = config_override.iter().map(|v| v.as_str()).collect();
        let r = apply_config(&strslice, r);
        assert!(r.is_ok());
        let r = r.unwrap();
        let a = r
            .spawn_config
            .control_config
            .get("a")
            .expect("a should still exist");
        if let ControllerType::Wasm(wasm) = a {
            assert_eq!(wasm.path, "foo.wasm");
            assert_eq!(wasm.fuel_per_update, None);
            assert_eq!(wasm.fuel_for_setup, None);
        } else {
            panic!("not of expected wasm type");
        };
    }
}
