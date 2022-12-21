
use std::fs::File;
use std::io::Read;

pub fn load_match_config(path: &str) -> Result<super::specification::ConstructConfig, Box<dyn std::error::Error>> {
    match File::open(path) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .expect("Should be able to read the file.");
            match serde_yaml::from_str(&content) {
                Ok(parsed_config) => Ok(parsed_config),
                Err(failure_message) => {
                    println!("Something went wrong parsing the configuration file:");
                    Err(Box::new(failure_message))
                }
            }
        }
        Err(error) => {
            Err(Box::new(error))
        }
    }
}
