use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{
        Read,
        Write,
    },
};
use dirs::home_dir;

#[allow(dead_code)]
#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    pub show_empty_channels: bool,
    pub mark_on_open: bool,
    pub down_on_mark: bool,
    pub app_title: String,
}

impl Config {
    pub fn default() -> Self {
        Config {
            show_empty_channels: true,
            mark_on_open: true,
            down_on_mark: true,
            app_title: String::from("TYT"),
        }
    }

    pub fn read_config_file() -> Config {
        let mut path = home_dir().unwrap();
        path.push(CONFIG_FILE_PATH);

        match File::open(path.clone()) {
            Ok(mut file) => {
                let mut reader = String::new();
                file.read_to_string(&mut reader).unwrap();
                let config: Config = match toml::from_str(&reader) {
                    Ok(file) => file,
                    Err(e) => panic!("could not parse config file: {}", e),
                };

                config
            },
            Err(_) => {
                match File::create(path) {
                    Ok(mut file) => {
                        let def_config = Config::default();
                        let string = toml::to_string(&def_config).unwrap();

                        match file.write_all(string.as_bytes()) {
                            Ok(_) => return Config::read_config_file(),
                            Err(e) => panic!("could not write default config: {}", e),
                        }
                    },
                    Err(e) => panic!("could not create config file: {}", e),
                };
            }
        }
    }
}

const CONFIG_FILE_PATH: &str = ".config/tyt/config";
