use battleground_construct::config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scenario_config = config::cli::parse_setup_args()?;
    let mut construct = config::setup::setup_scenario(scenario_config)?;

    let limit_max_time = 200.0;
    while !construct.is_match_finished() && (construct.elapsed_as_f32() < limit_max_time) {
        construct.update();
    }

    let wrap_up_config = config::cli::parse_wrap_up_args()?;
    let report = config::wrap_up::wrap_up_scenario(wrap_up_config, &construct)?;
    println!("{report:#?}");

    Ok(())
}
