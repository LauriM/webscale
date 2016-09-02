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

use webscale::{config, plugin};
use webscale::Session;
use std::path::Path;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    // Merge configuration from multiple sources.
    let filename = "config/webscale.toml";
    let mut config = match config::Config::load(filename) {
        Ok(loaded) => loaded,
        Err(err) => panic!(err.to_string())
    };

    // Initialize plugin container.
    let path = Path::new(&config.core.plugins);
    let mut registry = plugin::Registry::new();
    registry.scan(path);

    let mut sessions = Vec::new();
    for server in config.servers.clone() {
        sessions.push(thread::spawn(move || {
            let session = Session::new(&server).start();
        }));
    }

    for session in sessions {
        session.join().unwrap();
    }
}

