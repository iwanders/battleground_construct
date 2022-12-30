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

    /// Override properties from the configuration, the format of these keys is a bit bespoke.
    #[arg(short, long)]
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
            if super::reader::builtin_scenarios().contains(&scenario.scenario.as_str()) {
                return super::reader::get_builtin_scenario(&scenario.scenario);
            }

            // It wasn't... well, lets hope that it is a file...
            let p = std::path::PathBuf::from(&scenario.scenario);
            return super::reader::read_scenario_config(&p);
        }
    }
}
