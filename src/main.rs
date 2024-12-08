use crate::configuration::configuration_reader::read_json_configuration;
use serde::{Deserialize, Serialize};

mod configuration;

#[derive(Serialize, Deserialize, Debug)]
struct Person {
    name: String,
    age: u8,
}

fn main() {
    let config_path = ".test_data/person_test_data.json".to_string();
    match read_json_configuration::<Person>(config_path) {
        Ok(person_config) => {
            println!("{:#?}", person_config);
        }
        Err(error) => {
            eprintln!("Error: {}", error);
        }
    }
}
