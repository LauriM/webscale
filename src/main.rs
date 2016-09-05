#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use]
extern crate log;
extern crate toml;
extern crate semver;
extern crate serde;
extern crate libloading as lib;
extern crate irc;
extern crate getopts;
extern crate webscale_plugin;
extern crate glob;

mod webscale;

use getopts::Options;
use std::env;
use webscale::config::Config;
use webscale::plugin::Registry;
use webscale::Session;
use std::path::Path;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("c", "config", "configuration file location", "PATH");

    let options = match opts.parse(&args[1..]) {
        Ok(opts) => opts,
        Err(err) => panic!(err.to_string())
    };

    // Load configuration file from indicated source or default.
    let mut config_dir = std::env::current_dir().unwrap();
    match options.opt_str("c") {
        Some(path) => config_dir.push(path),
        None => config_dir.push("webscale.toml")
    };

    let config_path = config_dir.to_string_lossy().into_owned();
    println!("Loading config from: {:?}", config_path);
    let config = match Config::load(config_path) {
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

