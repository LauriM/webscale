#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use]
extern crate log;
extern crate toml;
extern crate semver;
extern crate serde;
extern crate libloading as lib;
extern crate irc;
extern crate webscale_plugin;
extern crate glob;

mod webscale;

use webscale::config::Config;
use webscale::plugin::Registry;
use webscale::Session;
use std::path::Path;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    // Merge configuration from multiple sources.
    let filename = "config/webscale.toml";
    let mut config = match Config::load(filename) {
        Ok(loaded) => loaded,
        Err(err) => panic!(err.to_string())
    };

    // Initialize plugin container.
    let path = Path::new(&config.core.plugins);
    let shared_registry = Arc::new(Mutex::new(Registry::new(path)));

    let mut sessions = Vec::new();
    for server in config.servers.clone() {
        let registry = shared_registry.clone();
        sessions.push(thread::spawn(move || {
            let session = Session::new(&server).start(registry);
        }));
    }

    for session in sessions {
        session.join().unwrap();
    }
}

