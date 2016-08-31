#![feature(custom_derive, plugin)]

#[macro_use]
extern crate log;
extern crate toml;
extern crate semver;
extern crate serde;
extern crate glob;
extern crate webscale_plugin;

mod webscale;

use webscale::{config, plugin};
use std::path::Path;

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

    println!("{:?}", config);
}

