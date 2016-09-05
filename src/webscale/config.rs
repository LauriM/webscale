use std::fs::File;
use std::io::Read;
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize, Deserializer};
use toml;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub core: CoreConfig,
    pub servers: Vec<ServerConfig>,
    pub plugins: toml::Table
}

impl Config {
    pub fn load(filename: &str) -> Result<Config, String> {
        let mut file = match File::open(filename) {
            Ok(file) => file,
            Err(err) => return Err(err.to_string())
        };

        let mut content = String::new();
        if let Err(err) = file.read_to_string(&mut content) {
            return Err(err.to_string());
        }

        let mut parser = toml::Parser::new(&content);
        let mut decoder = match parser.parse() {
            Some(toml) => toml::Decoder::new(toml::Value::Table(toml)),
            None => return Err("The config file was empty.".to_string())
        };

        match Deserialize::deserialize(&mut decoder) {
            Ok(config) => Ok(config),
            Err(err) => Err(err.to_string())
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct CoreConfig {
    pub retries: i32,
    pub plugins: String
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub nickname: String,
    pub hostname: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub ssl: Option<bool>,
    pub channels: Option<Vec<String>>
}
