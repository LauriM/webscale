#[derive(Debug, Deserialize)]
pub struct Config {
    pub core: CoreConfig,
    pub servers: Vec<ServerConfig>,
    pub plugins: toml::Value
}

#[derive(Debug, Deserialize)]
pub struct CoreConfig {
    pub watch: bool,
    pub plugins: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub nickname: String,
    pub hostname: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub ssl: Option<bool>,
    pub channels: Option<Vec<String>>
}
