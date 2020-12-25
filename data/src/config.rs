use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{
        Read,
        Write,
    },
};
use dirs::home_dir;

const CONFIG_FILE_PATH: &str = ".config/tyt/config";

#[allow(dead_code)]
#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    pub show_empty_channels: bool,
    #[serde(default = "default_bool_true")]
    pub mark_on_open: bool,
    #[serde(default = "default_bool_true")]
    pub down_on_mark: bool,
    #[serde(default = "default_title")]
    pub app_title: String,
    #[serde(default = "default_bool_true")]
    pub update_at_start: bool,
    #[serde(default = "default_bool_false")]
    pub sort_by_tag: bool,
    #[serde(default = "default_20")]
    pub message_timeout: u8,
    #[serde(default = "default_bool_true")]
    pub use_notify_send: bool,
}

fn default_title() -> String {
    String::from("TYT")
}

fn default_bool_true() -> bool {
    true
}

fn default_bool_false() -> bool {
    false
}

fn default_20() -> u8 {
    20
}

impl Config {
    pub fn default() -> Self {
        Config {
            show_empty_channels: true,
            mark_on_open: true,
            down_on_mark: true,
            app_title: String::from("TYT"),
            update_at_start: true,
            sort_by_tag: false,
            message_timeout: 20,
            use_notify_send: true,
        }
    }

    pub fn read_config_file() -> Self {
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
