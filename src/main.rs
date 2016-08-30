#[macro_use]
extern crate log;
extern crate toml;
extern crate rustc_serialize;

mod webscale;

use webscale::config;

fn main() {
    // Merge configuration from multiple sources.
    let filename = "config/webscale.toml";
    let mut config = match config::Config::load(filename) {
        Ok(loaded) => loaded,
        Err(err) => panic!(err.to_string())
    };

    println!("{:?}", config);
}

