use std::collections::HashMap;
use std::path::Path;
use glob::glob;
use semver::Version;
use webscale::config::Config;

#[cfg(windows)]
const EXTENSION: &'static str = "dll";

#[cfg(mac)]
const EXTENSION: &'static str = "dylib";

#[cfg(unix)]
const EXTENSION: &'static str = "so";

pub struct Registry {
    index: HashMap<String, Status>
}

impl Registry {
    pub fn new() -> Self {
        Registry { index: HashMap::new() }
    }

    pub fn scan(&mut self, path: &Path) {
        let mut route_buf = path.to_path_buf();
        route_buf.push("*");
        route_buf.set_extension(EXTENSION);

        let route = route_buf.to_str().unwrap();

        for entry in glob(route).unwrap() {
            if let Ok(resolved) = entry {
                println!("{:?}", resolved.display());
            }
        }
    }
}

enum Status {
    Failed(String),
    Loaded(Handle)
}

pub struct Handle {
    name: String,
    version: Version,
    source: String
}
