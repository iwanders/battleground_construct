use std::fs::File;
use std::io::Read;

pub fn read_scenario_config(
    path: &std::path::Path,
) -> Result<super::specification::ScenarioConfig, Box<dyn std::error::Error>> {
    match File::open(path) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .expect("should be able to read the file.");
            load_yaml_config(&content)
        }
        Err(error) => Err(Box::<dyn std::error::Error>::from(format!(
            "failed to open {}: {}",
            path.display(),
            error,
        ))),
    }
}

fn load_yaml_config(
    content: &str,
) -> Result<super::specification::ScenarioConfig, Box<dyn std::error::Error>> {
    match serde_yaml::from_str(content) {
        Ok(parsed_config) => Ok(parsed_config),
        Err(failure_message) => Err(Box::new(failure_message)),
    }
}

static BUILTINS_SCENARIO: [(&str, &[u8]); 7] = [
    ("test", include_bytes!("scenario/test.yaml")),
    ("playground", b"pre_setup: playground\n"),
    (
        "tutorial_01_driving_forward",
        include_bytes!("scenario/tutorial_01_driving_forward.yaml"),
    ),
    (
        "tutorial_02_shoot",
        include_bytes!("scenario/tutorial_02_shoot.yaml"),
    ),
    (
        "tutorial_03_driving_objective",
        include_bytes!("scenario/tutorial_03_driving_objective.yaml"),
    ),
    (
        "tutorial_04_shoot_radar",
        include_bytes!("scenario/tutorial_04_shoot_radar.yaml"),
    ),
    (
        "tutorial_05_shoot_enemies",
        include_bytes!("scenario/tutorial_05_shoot_enemies.yaml"),
    ),
];

pub fn get_builtin_scenario(
    desired_name: &str,
) -> Result<super::specification::ScenarioConfig, Box<dyn std::error::Error>> {
    for (name, scenario) in BUILTINS_SCENARIO.iter() {
        if desired_name == *name {
            let v = std::str::from_utf8(scenario).unwrap();
            return load_yaml_config(v);
        }
    }
    Err(Box::<dyn std::error::Error>::from(format!(
        "builtin scenario named {} does not exist",
        desired_name
    )))
}

pub fn builtin_scenarios() -> &'static [(&'static str, &'static [u8])] {
    &BUILTINS_SCENARIO
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_scenario_readable() {
        for v in builtin_scenarios() {
            assert!(get_builtin_scenario(v.0).is_ok());
        }
    }
}
