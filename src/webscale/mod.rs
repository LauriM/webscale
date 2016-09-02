pub mod config;
pub mod plugin;

use irc::client::prelude::*;
use std::default::Default;
use webscale::plugin::Registry;
use webscale::config::ServerConfig;

pub struct Session { 
    server: IrcServer
}

impl Session {
    pub fn new(source: &ServerConfig) -> Session {
        let config = Config {
            nickname: Some(source.nickname.clone()),
            server: Some(source.hostname.clone()),
            port: source.port,
            use_ssl: source.ssl,
            channels: source.channels.clone(),
            .. Default::default()
        };

        println!("{:?}", config);

        Session { 
            server: IrcServer::from_config(config).unwrap()
        }
    }

    pub fn start(&self) {
        self.server.identify().unwrap();

        for message in self.server.iter() {
            println!("{:?}", message);
        }
    }
}

