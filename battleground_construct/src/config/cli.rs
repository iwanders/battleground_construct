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

/// Battleground Construct
#[derive(Debug, Args)]
struct Scenario {
    /// Scenario to load, can be one of the builtins, or a yaml file specifying the scenario.
    #[arg(value_hint = clap::ValueHint::DirPath)]
    scenario: String,

    /// List built in scenario's and quit.
    #[arg(long)]
    list: bool,

    /// Override properties from the configuration, the format of these keys is a bit bespoke.
    #[arg(short, long)]
    config: Vec<String>,
}

pub fn parse_args() -> Result<ScenarioConfig, Box<dyn std::error::Error>> {
    let args = Cli::parse();
    println!("args: {args:?}");
    Ok(Default::default())
}
