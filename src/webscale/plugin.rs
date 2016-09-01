use std::collections::{HashMap, BTreeMap};
use std::path::Path;
use glob::glob;
use semver::Version;
use lib::{Library, Symbol};
use webscale::config::Config;
use webscale_plugin::{Plugin, PluginConfig, PluginDescription};

#[cfg(target_os = "windows")]
const EXTENSION: &'static str = "dll";

#[cfg(target_os = "macos")]
const EXTENSION: &'static str = "dylib";

#[cfg(target_os = "unix")]
const EXTENSION: &'static str = "so";

const DESCRIPTION_LABEL: &'static [u8] = b"WS_PLUGIN_DESCRIPTION\0";

type PluginStatus = Result<Handle, String>;
type Initializer = unsafe extern fn(&PluginConfig) -> Box<Plugin>;

pub struct Registry {
    index: HashMap<String, PluginStatus>
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
        let libraries = glob(route).unwrap().filter_map(|r| {
            match r {
                Ok(res) => Some(res),
                Err(_) => None
            }
        });
        
        for resolved in libraries {
            let raw_path = resolved.to_str().unwrap();
            println!("Loading plugin from {}.", raw_path);
            self.index.insert(String::from(raw_path), Self::load(raw_path));
        }
    }

    fn load(path: &str) -> PluginStatus {
        let lib = match Library::new(path) {
            Ok(loaded) => loaded,
            Err(err) => return Err(err.to_string())
        };

        unsafe {
            let description: Symbol<*mut PluginDescription> = 
                match lib.get(DESCRIPTION_LABEL) {
                    Ok(desc) => desc,
                    Err(err) => {
                        println!("Failed to load description from {}.", path);
                        return Err(err.to_string());
                    }
                };

            println!("Found plugin description {:?}.", **description);
            let initializer: Initializer = 
                match lib.get((**description).initializer) {
                    Ok(func) => *func,
                    Err(err) => {
                        println!("Failed to locate initializer for {}.", path);
                        return Err(err.to_string());
                    }
                };

            Ok(Handle {
                name: String::from((**description).name),
                version: Version::parse((**description).version).unwrap(),
                source: path.to_string(),
                plugin: initializer(&BTreeMap::new())
            })
        }
    }
}

pub struct Handle {
    name: String,
    version: Version,
    source: String,
    plugin: Box<Plugin>
}
