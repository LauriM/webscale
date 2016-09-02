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
const INITIALIZER_LABEL: &'static [u8] = b"initialize_plugin\0";

type PluginStatus = Result<Handle, String>;
type Initializer = unsafe extern fn(&PluginConfig) -> Box<Plugin>;

pub struct Registry {
    index: HashMap<String, PluginStatus>,
    lookup: HashMap<String, String>
}

impl Registry {
    pub fn new() -> Self {
        Registry { index: HashMap::new(), lookup: HashMap::new() }
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

            let status = Self::load(raw_path);
            if let Ok(handle) = status {
                self.lookup.insert(String::from(raw_path), handle.name.clone());
                self.index.insert(String::from(raw_path), Ok(handle));
            } else {
                self.index.insert(String::from(raw_path), status);
            }
        }
    }

    fn load(path: &str) -> PluginStatus {
        let lib = match Library::new(path) {
            Ok(loaded) => loaded,
            Err(err) => return Err(err.to_string())
        };

        let (name, version, plugin) = unsafe {
            let description: Symbol<*mut PluginDescription> = 
                match lib.get(DESCRIPTION_LABEL) {
                    Ok(desc) => desc,
                    Err(err) => {
                        println!("Failed to load description from {}.", path);
                        return Err(err.to_string());
                    }
                };

            println!("Found plugin description {:?}.", **description);
            let initializer: Symbol<Initializer> = 
                match lib.get(INITIALIZER_LABEL) {
                    Ok(func) => func,
                    Err(err) => {
                        println!("Failed to locate initializer {:?} for {}.", 
                                 (**description).initializer, path);
                        return Err(err.to_string());
                    }
                };

            let name = String::from((**description).name);
            let version = Version::parse((**description).version).unwrap();
            let initializer = initializer;
            let plugin = initializer(&BTreeMap::new()) as Box<Plugin>;
            
            (name, version, plugin)
        };

        Ok(Handle {
            name: name,
            version: version,
            source: path.to_string(),
            plugin: plugin,
            library: lib
        })
    }
}

pub struct Handle {
    name: String,
    version: Version,
    source: String,
    plugin: Box<Plugin>,
    library: Library
}
