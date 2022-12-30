use battleground_construct::config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scenario_config = config::cli::parse_args()?;
    let mut construct = config::setup::setup_scenario(scenario_config)?;

    let max_time = 20.0;
    while construct.elapsed_as_f32() < max_time {
        construct.update();
    }

    Ok(())
}
