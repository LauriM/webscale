use std::fs::File;
use std::io::Read;
use serde::Deserialize;
use Config;
use toml;

impl Config {
    pub fn load(filename: String) -> Result<Config, String> {
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

