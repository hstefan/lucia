use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::str;

use crate::common::Result;
const FILENAME: &str = "lucia.json";
const APP_NAME: &str = "lucia";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub app_name: String,
    pub user_name: Option<String>,
    pub client_key: Option<String>,
    pub bridge_ip: Option<String>,
}

fn find_config_path() -> PathBuf {
    let home = env!("HOME");
    Path::new(home).join(FILENAME)
}

fn gen_app_name() -> String {
    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_owned());
    format!("{}#{}", APP_NAME, user)
}

impl Config {
    pub fn load() -> Result<Config> {
        let path = find_config_path();
        if path.exists() {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            Ok(serde_json::from_reader(reader)?)
        } else {
            Ok(Config {
                app_name: gen_app_name(),
                user_name: None,
                client_key: None,
                bridge_ip: None,
            })
        }
    }

    pub fn persist(&self) -> Result<()> {
        let contents = serde_json::to_string(self)?;
        let mut file = File::create(find_config_path())?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }
}
