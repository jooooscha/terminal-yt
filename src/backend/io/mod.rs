use dirs_next::home_dir;
use std::{
    fs::{OpenOptions, create_dir_all},
    path::PathBuf,
    io::{Read, Write, ErrorKind},
};
use crate::{
    notification::notify_error,
    backend::{
        io::{
            config::Config,
            subscriptions::Subscriptions,
            history::History,
        },
        data::channel_list::ChannelList,
    },
};

pub(crate) mod subscriptions;
pub(crate) mod config;
pub(crate) mod history;

const CONFIG_PATH: &str = ".config/tyt/";

const CONFIG_FILE: &str = "config.yml";

#[cfg(not(debug_assertions))]
const DB_FILE: &str = "db.json";
#[cfg(not(debug_assertions))]
const HISTORY_FILE: &str = "history.json";
#[cfg(not(debug_assertions))]
const SUBSCRIPTIONS_FILE: &str = "subscriptions.yml";

#[cfg(debug_assertions)]
const DB_FILE: &str = "db_debug.json";
#[cfg(debug_assertions)]
const HISTORY_FILE: &str = "history_debug.json";
#[cfg(debug_assertions)]
const SUBSCRIPTIONS_FILE: &str = "subscriptions_debug.yml";

#[allow(clippy::enum_variant_names)]
#[derive(PartialEq)]
pub(crate) enum FileType {
    ConfigFile,
    DbFile,
    HistoryFile,
    SubscriptionsFile,
}

impl FileType {
    fn file(&self) -> &str {
        match self {
            FileType::ConfigFile => CONFIG_FILE,
            FileType::DbFile => DB_FILE,
            FileType::HistoryFile => HISTORY_FILE,
            FileType::SubscriptionsFile => SUBSCRIPTIONS_FILE,
        }
    }

    // writes and returns default
    fn write_default(self) -> String {
        let string = match self {
            FileType::ConfigFile => {
                serde_yaml::to_string(&Config::default()).unwrap()
            }
            FileType::DbFile => {
                serde_json::to_string(&ChannelList::default()).unwrap()
            }
            FileType::HistoryFile => {
                serde_json::to_string(&History::default()).unwrap()
            }
            FileType::SubscriptionsFile => {
                serde_yaml::to_string(&Subscriptions::default()).unwrap()
            }
        };

        write_config(self, &string);
        string
    }
}

/// Read in config dir
pub(crate) fn read_config(file_type: FileType) -> String {

    let config_dir = get_config_dir();

    let file_path = config_dir.join(file_type.file());

    let file_result = OpenOptions::new()
        .read(true)
        .open(file_path);

    // return content (or default on error)
    match file_result {
        Ok(mut file) => {
            let mut buffer = String::new();
            if let Err(e) = file.read_to_string(&mut buffer) {
                notify_error(&format!("Data is no valid utf-8: {}", e));
            }

            buffer
        }
        Err(error) => {
            match error.kind() {
                ErrorKind::NotFound => {
                    file_type.write_default()
                },
                other_error => {
                    notify_error(&format!("Could not read file: {:?}", other_error));
                    panic!("Could not read file: {:?}", other_error);
               },
            }
        },
    }
}

/// Write in config dir
pub(crate) fn write_config(r#type: FileType, content: &str) {
    let config_dir = get_config_dir();

    let file_path = config_dir.join(r#type.file());

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)
        .unwrap();

    // if r#type == FileType::DbFile {
    //     eprintln!("######");
    //     eprintln!("content: {:?}", content);
    //     eprintln!("######");
    // }
    let _ = file.write_all(content.as_bytes());
}

// private function to create and read config dir
fn get_config_dir() -> PathBuf {
    // crate config dir if not exists
    let home_dir = match home_dir() {
        Some(p) => p,
        None => {
            let error = "could not read home dir";
            notify_error(error);
            panic!("{}", error);
        }
    };
    let path = home_dir.join(CONFIG_PATH);
    if let Err(error) = create_dir_all(&path) {
        if error.kind() == ErrorKind::PermissionDenied {
                notify_error("Permission to config dir denied");
        }
    }

    path
}
