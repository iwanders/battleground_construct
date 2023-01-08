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

static SCENARIO_TEST: &[u8] = include_bytes!("scenario/test.yaml");
static SCENARIO_PLAYGROUND: &[u8] = b"pre_setup: playground\n";
static NAME_TEST: &str = "test";
pub static NAME_PLAYGROUND: &str = "playground";
static BUILTINS: [&str; 2] = [NAME_TEST, NAME_PLAYGROUND];

pub fn get_builtin_scenario(
    name: &str,
) -> Result<super::specification::ScenarioConfig, Box<dyn std::error::Error>> {
    if name == NAME_TEST {
        let v = std::str::from_utf8(SCENARIO_TEST).unwrap();
        return load_yaml_config(v);
    }
    if name == NAME_PLAYGROUND {
        let v = std::str::from_utf8(SCENARIO_PLAYGROUND).unwrap();
        return load_yaml_config(v);
    }
    Err(Box::<dyn std::error::Error>::from(format!(
        "builtin scenario named {} does not exist",
        name
    )))
}

pub fn builtin_scenarios() -> &'static [&'static str] {
    &BUILTINS
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_scenario_readable() {
        for v in builtin_scenarios() {
            assert!(get_builtin_scenario(v).is_ok());
        }
    }
}
