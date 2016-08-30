use std::fs::File;
use std::io::Read;
use std::collections::BTreeMap;
use rustc_serialize::Decodable;
use toml;

#[derive(Debug, RustcDecodable, Default)]
pub struct Config {
    pub core: CoreConfig,
    pub server: Vec<ServerConfig>,
    pub plugin: BTreeMap<String, toml::Value>
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

        match Config::decode(&mut decoder) {
            Ok(config) => Ok(config),
            Err(err) => Err(err.to_string())
        }
    }
}

#[derive(Debug, RustcDecodable, Default)]
pub struct CoreConfig {
    pub retries: i32,
    pub plugins: String
}

#[derive(Debug, RustcDecodable, Default)]
pub struct ServerConfig {
    pub name: String,
    pub hostname: String,
    pub port: i32
}
