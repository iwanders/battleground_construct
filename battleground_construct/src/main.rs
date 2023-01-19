use battleground_construct::config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = config::cli::parse_args()?;
    let setup_config = config::cli::command_to_setup(&command)?;
    let mut construct = config::setup::setup(&setup_config)?;

    let limit_max_time = 200.0;
    while !construct.is_match_finished() && (construct.elapsed_as_f32() < limit_max_time) {
        construct.update();
    }

    let wrap_up_config = config::cli::command_to_wrap_up(&command)?;
    let report = config::wrap_up::wrap_up_scenario(wrap_up_config, &mut construct)?;
    println!("{report:#?}");

    Ok(())
}
