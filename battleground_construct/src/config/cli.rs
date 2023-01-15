use super::specification::{ScenarioConfig, WrapUpConfig};
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
    #[command(arg_required_else_help = true)]
    Play(Play),
    #[command(arg_required_else_help = true)]
    #[command(subcommand)]
    Recording(RecordingCommands),
}

/// Play subcommand
#[derive(Debug, Args)]
struct Play {
    /// Path to use
    #[arg(value_hint = clap::ValueHint::FilePath)]
    file: String,
}

/// Play subcommand
#[derive(Debug, Args)]
struct Seek {
    /// Path to use
    #[arg(value_hint = clap::ValueHint::FilePath)]
    file: String,

    time: f32,
}

/// Commands related to recordings
#[derive(Subcommand, Debug)]
enum RecordingCommands {
    /// Specify the scenario to run.
    #[command(arg_required_else_help = true)]
    Analyze(Play),
    #[command(arg_required_else_help = true)]
    Seek(Seek),
}

/// Scenario subcommand
#[derive(Debug, Args)]
struct Scenario {
    /// Scenario to load, can be one of the builtins, or a yaml file specifying the scenario. Use
    /// 'list' as a special scenario to list the builtins.
    #[arg(value_hint = clap::ValueHint::FilePath)]
    scenario: String,

    #[cfg(feature = "unit_control_wasm")]
    #[arg(long, verbatim_doc_comment)]
    /// Direct override of the path attribute of wasm controllers in the controller configs.
    /// Use with --wasm team_a:path_to_module.wasm --wasm team_b:path_to_module.wasm.
    wasm: Vec<String>,

    #[cfg(feature = "unit_control_wasm")]
    #[arg(long, verbatim_doc_comment)]
    /// Direct override of the path attribute of wasm controllers in the team configs.
    /// Use with --team team_a:path_to_module.wasm --team team_b:path_to_module.wasm.
    /// Also sets the comment to the basename of the path.
    team: Vec<String>,

    /// Override properties from the configuration, the format of these keys is a bit bespoke and verbose.
    /// In general, it is prefered to use the --wasm argument.
    /// It is limited to (depending on enabled controllers)
    /// - "control:controller_name:wasm:path:foo.wasm" -> Set controller by 'controller_name' to wasm and use 'foo.wasm'.
    /// - "control:controller_name:wasm:fuel_per_update:1000"  (or none) -> Set fuel_per_update to none or value.
    /// - "control:controller_name:wasm:fuel_for_setup:1000"  (or none) -> Set fuel_for_setup to none or value.
    /// - "control:controller_name:none" -> Set the controller with 'controller_name' to none.
    #[arg(short, long, verbatim_doc_comment)]
    config: Vec<String>,

    /// After the match concludes, write a report yaml file to this path.
    #[arg(short, long)]
    report: Option<String>,

    /// Record the match to this path.
    #[arg(short = 'w', long)]
    record: Option<String>,

    /// Outro duration, defaults to 4.55 seconds.
    #[arg(long)]
    outro_duration: Option<f32>,

    /// Overwrite or apply the time limit.
    #[arg(short = 'l', long)]
    time_limit: Option<f32>,
}

/// This creates a config struct handled by the wrap up functionality
pub fn parse_wrap_up_args() -> Result<WrapUpConfig, Box<dyn std::error::Error>> {
    let setup = parse_setup_args()?;
    let args = Cli::parse();

    let write_wrap_up = match args.command {
        Commands::Scenario(ref scenario) => &scenario.report,
        _ => &None,
    }
    .clone();
    let write_recording = match args.command {
        Commands::Scenario(ref scenario) => &scenario.record,
        _ => &None,
    }
    .clone();

    let outro = match args.command {
        Commands::Scenario(ref scenario) => scenario.outro_duration.unwrap_or(4.55),
        _ => 0.0,
    };

    let scenario = match setup {
        Setup::Scenario(config) => Some(config),
        _ => None,
    };

    Ok(WrapUpConfig {
        scenario,
        write_wrap_up,
        write_recording,
        outro,
    })
}

pub enum Setup {
    Scenario(ScenarioConfig),
    Play(String),
}

pub fn parse_setup_args() -> Result<Setup, Box<dyn std::error::Error>> {
    let args = Cli::parse();
    match args.command {
        Commands::Scenario(scenario) => {
            if scenario.scenario == "list" {
                let available = super::reader::builtin_scenarios();
                println!("Available scenarios:");
                for name in available {
                    println!("  {}", name.0);
                }
                std::process::exit(0);
            }

            // Check if it is a built in scenario
            let mut specification =
                if super::reader::builtin_scenarios().iter().map(|x| x.0).collect::<Vec<_>>().contains(&scenario.scenario.as_str()) {
                    super::reader::get_builtin_scenario(&scenario.scenario)?
                } else {
                    // It wasn't... well, lets hope that it is a file...
                    let p = std::path::PathBuf::from(&scenario.scenario);
                    super::reader::read_scenario_config(&p)?
                };

            if let Some(new_limit) = scenario.time_limit {
                specification.match_config.time_limit = Some(new_limit);
            }

            if scenario.record.is_some() {
                specification.recording = true;
            }

            #[cfg(not(feature = "unit_control_wasm"))]
            let extra_config: Vec<String> = vec![];

            #[cfg(feature = "unit_control_wasm")]
            let extra_config = {
                let mut extra_config = vec![];
                for wasm in scenario.wasm.iter() {
                    let split = wasm
                        .split(':')
                        .map(|v| v.to_owned())
                        .collect::<Vec<String>>();
                    if split.len() != 2 {
                        return Err(Box::<dyn std::error::Error>::from(
                            "expected ':' between controller name and path",
                        ));
                    }
                    extra_config.push(format!("control:{}:wasm:path:{}", split[0], split[1]));
                }
                for team_entry in scenario.team.iter() {
                    let split = team_entry
                        .split(':')
                        .map(|v| v.to_owned())
                        .collect::<Vec<String>>();
                    if split.len() != 2 {
                        return Err(Box::<dyn std::error::Error>::from(
                            "expected ':' between team name and path",
                        ));
                    }
                    extra_config.push(format!("team:{}:control:wasm:path:{}", split[0], split[1]));
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
            apply_config(&config_strs, specification).map(Setup::Scenario)
        }
        Commands::Play(play) => Ok(Setup::Play(play.file)),
        Commands::Recording(subcommand) => {
            recording_subcommand_handler(subcommand)?;
            Err("done".into())
        }
    }
}

fn recording_subcommand_handler(cmd: RecordingCommands) -> Result<(), Box<dyn std::error::Error>> {
    use crate::components::recording::Recording;
    match cmd {
        RecordingCommands::Analyze(z) => {
            let mut total = 0;
            let v = Recording::load_file(&z.file)?;
            let record = v.record();
            for (name, count) in record.borrow().get_byte_sums() {
                println!("{name: <30}{count: >30}");
                total += count;
            }
            println!("{name: <30}{count: >30}", name = "total", count = total);
        }
        RecordingCommands::Seek(z) => {
            let v = Recording::load_file(&z.file)?;
            let record = v.record();
            println!("starting seek");
            record.borrow_mut().seek(z.time);
            println!("finished seek");
        }
    }
    Ok(())
}

use crate::config::specification::ControllerType;
fn controller_config(
    config: &str,
    control: &mut ControllerType,
    comment: Option<&mut String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let make_error = |s: &str| Box::<dyn std::error::Error>::from(s);

    let _comment = &comment; // Prevent unused variable for comment in non-wasm.

    let mut tokens = config.split(':');
    let control_type = tokens
        .next()
        .ok_or_else(|| make_error("expected control type"))?;
    match control_type {
        #[cfg(feature = "unit_control_wasm")]
        "wasm" => {
            let wasm = if let ControllerType::Wasm(ref mut v) = control {
                v
            } else {
                if !matches!(control, ControllerType::Wasm(_)) {
                    *control = ControllerType::Wasm(Default::default());
                } else {
                    println!("Unit control is already wasm");
                }
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
                    let path_string = tokens
                        .next()
                        .ok_or_else(|| make_error("expected path to file"))?;
                    wasm.path = path_string.to_owned();

                    // we got a path, conver
                    let path = std::path::Path::new(&path_string);
                    let fname = path
                        .file_name()
                        .map(|v| v.to_str())
                        .unwrap_or_else(|| Some(path_string))
                        .ok_or_else(|| make_error("expected path to file"))?;
                    if let Some(v) = comment {
                        *v = fname.to_owned();
                    }
                }
                "fuel_per_update" => {
                    let value_str = tokens
                        .next()
                        .ok_or_else(|| make_error("expected value for fuel_per_update"))?;
                    let value = if value_str.to_lowercase() == "none" {
                        None
                    } else {
                        Some(value_str.parse::<u64>()?)
                    };
                    wasm.fuel_per_update = value;
                }
                "fuel_for_setup" => {
                    let value_str = tokens
                        .next()
                        .ok_or_else(|| make_error("expected value for fuel_for_setup"))?;
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
        "idle" => {
            *control = ControllerType::Idle;
        }
        _ => {
            return Err(make_error(
                format!("cannot handle control {} overrides", control_type).as_str(),
            ));
        }
    }
    Ok(())
}

// Well, this function is a bit... much.
fn apply_config(
    config: &[&str],
    scenario: ScenarioConfig,
) -> Result<ScenarioConfig, Box<dyn std::error::Error>> {
    let mut scenario = scenario;
    let make_error = |s: &str| Box::<dyn std::error::Error>::from(s);

    for c in config.iter() {
        let mut tokens = c.split(':');

        match tokens.next().ok_or_else(|| make_error("expected token"))? {
            "team" => {
                // team:{}:control:wasm:path:{}
                let section = &mut scenario.spawn_config.teams;
                let key = tokens
                    .next()
                    .ok_or_else(|| make_error("expected team name"))?;
                let pos = section
                    .iter()
                    .position(|x| x.name.to_lowercase() == key.to_lowercase())
                    .ok_or_else(|| {
                        make_error(&format!(
                            "couldnt find team name {}; {:?}",
                            key,
                            section.iter().map(|x| &x.name).collect::<Vec<_>>()
                        ))
                    })?;
                let team = &mut section[pos];
                let team_attribute = tokens
                    .next()
                    .ok_or_else(|| make_error("expected team attribute to modify"))?;
                match team_attribute {
                    "control" => {
                        // populate it with Idle, them use the common handler.
                        if team.controller.is_none() {
                            team.controller = Some(ControllerType::Idle);
                        }
                        let controller = team.controller.as_mut().unwrap();
                        team.comment = Some("".to_owned());
                        controller_config(
                            &tokens.collect::<Vec<_>>().join(":"),
                            controller,
                            team.comment.as_mut(),
                        )?;
                    }
                    unhandled_key => {
                        return Err(make_error(
                            format!("cannot handle team attribute {} overrides", unhandled_key)
                                .as_str(),
                        ));
                    }
                }
            }
            "control" => {
                let section = &mut scenario.spawn_config.control_config;
                let key = tokens
                    .next()
                    .ok_or_else(|| make_error("expected control key"))?;
                let control = section
                    .get_mut(key)
                    .ok_or_else(|| make_error(format!("key {} didnt exist", key).as_str()))?;

                controller_config(&tokens.collect::<Vec<_>>().join(":"), control, None)?;
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

    #[cfg(feature = "unit_control_wasm")]
    #[test]
    fn test_config_cli_team_config() {
        use crate::config::specification::ControllerType;
        use crate::config::specification::SpawnConfig;
        use crate::config::specification::Team;
        let v = ScenarioConfig {
            spawn_config: SpawnConfig {
                teams: vec![
                    Team {
                        name: "red".to_owned(),
                        color: (255, 0, 0),
                        controller: None,
                        comment: None,
                    },
                    Team {
                        name: "blue".to_owned(),
                        color: (0, 255, 0),
                        controller: None,
                        comment: None,
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        };

        let config_override: Vec<String> = vec![
            "team:red:control:wasm:path:foo.wasm".to_owned(),
            "team:blue:control:wasm:path:bar.wasm".to_owned(),
        ];
        let strslice: Vec<&str> = config_override.iter().map(|v| v.as_str()).collect();
        let r = apply_config(&strslice, v);
        assert!(r.is_ok());
        let r = r.unwrap();
        let team_a = &r.spawn_config.teams.get(0).expect("team 0 still exist");
        let controller_a = &team_a.controller;
        let controller_a = controller_a.as_ref().expect("should now have a controller");
        assert_eq!(team_a.comment, Some("foo.wasm".to_owned()));
        if let ControllerType::Wasm(wasm) = controller_a {
            assert_eq!(wasm.path, "foo.wasm");
        } else {
            panic!("not of expected wasm type");
        };

        let team_b = &r.spawn_config.teams.get(1).expect("team 0 still exist");
        let controller_b = &team_b.controller;
        let controller_b = controller_b.as_ref().expect("should now have a controller");
        assert_eq!(team_b.comment, Some("bar.wasm".to_owned()));
        if let ControllerType::Wasm(wasm) = controller_b {
            assert_eq!(wasm.path, "bar.wasm");
        } else {
            panic!("not of expected wasm type");
        };
    }
}
