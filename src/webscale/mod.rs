pub mod config;
pub mod plugin;

use irc::client::prelude::*;
use std::default::Default;
use webscale::plugin::Registry;
use std::sync::{Arc, Mutex};
use webscale_plugin::Link;
use ServerConfig;

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

    pub fn start(&self, registry: Arc<Mutex<Registry>>) {
        self.server.identify().unwrap();

        // Notify all plugins that this session has started.
        {
            let registry = registry.lock().unwrap();
            for status in registry.iter() {
                if let &Ok(ref handle) = status {
                    handle.plugin.on_connect(self);
                }
            }
        }

        // Poll messages from the server and dispatch them to
        // registered plugins.
        for message in self.server.iter() {
            let message = message.unwrap();
            let registry = registry.lock().unwrap();

            // Handle system-level queries.
            if let Command::PRIVMSG(ref target, ref content) = message.command {
                match content.as_str() {
                    "!status" => {
                        for row in registry.to_string().split("\n") {
                            self.send(target, row);
                        }
                    }
                    _ => ()
                };
            }

            println!("{:?}", message);
            for status in registry.iter() {
                if let &Ok(ref handle) = status {
                    match message.command {
                        Command::PRIVMSG(ref target, ref content) => {
                            handle.plugin.on_message(self, target, content);
                        },
                        _ => ()
                    }
                }
            }
        }
    }
}

impl Link for Session {
    fn send(&self, target: &str, message: &str) {
        self.server.send_privmsg(target, message);
    }
}

