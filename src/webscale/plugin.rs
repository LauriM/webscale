use std::collections::HashMap;
use std::collections::hash_map;
use std::path::Path;
use glob::glob;
use semver::Version;
use lib::{Library, Symbol};
use toml;
use prettytable::row::Row;
use prettytable::cell::Cell;
use std::fmt;
use webscale_plugin::{Plugin, ConfigSource, PluginDescription};

#[cfg(target_os = "windows")]
const EXTENSION: &'static str = "dll";

#[cfg(target_os = "macos")]
const EXTENSION: &'static str = "dylib";

#[cfg(target_os = "linux")]
const EXTENSION: &'static str = "so";

const DESCRIPTION_LABEL: &'static [u8] = b"WS_PLUGIN_DESCRIPTION\0";

pub type PluginStatus = Result<Handle, String>;
pub type Initializer = unsafe extern fn(Box<ConfigSource>) -> Box<Plugin>;

pub struct Registry {
    index: HashMap<String, PluginStatus>,
    lookup: HashMap<String, String>
}

impl Registry {
    pub fn new(path: &Path, config: &toml::Value) -> Self {
        let mut index = HashMap::new();
        let mut lookup = HashMap::new();

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

            let status = Self::load(raw_path, config);
            if let Ok(handle) = status {
                lookup.insert(String::from(raw_path), handle.name.clone());
                index.insert(String::from(raw_path), Ok(handle));
            } else {
                index.insert(String::from(raw_path), status);
            }
        }

        Registry { lookup: lookup, index: index }
    }

    fn load(path: &str, config: &toml::Value) -> PluginStatus {
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
                match lib.get((**description).initializer) {
                    Ok(func) => func,
                    Err(err) => {
                        println!("Failed to locate initializer {:?} for {}.", 
                                 (**description).initializer, path);
                        return Err(err.to_string());
                    }
                };

            let name = String::from((**description).name);
            let version = Version::parse((**description).version).unwrap();

            let config = match config.lookup(&(String::from("plugins.") + &name)) {
                Some(value) => PluginConfig::new(value.clone()),
                None => PluginConfig::new(toml::Value::Table(toml::Table::new()))
            };
            let config = Box::new(config) as Box<ConfigSource>;
            let plugin = initializer(config) as Box<Plugin>;
            
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

    pub fn iter<'a>(&'a self) -> RegistryIterator {
        RegistryIterator::new(self)
    }

    pub fn get_plugin(&self, name: &str) -> Option<&Box<Plugin>> {
        let path = match self.lookup.get(name) {
            Some(path) => path,
            None => return None
        };

        self.get_plugin_from(path)
    }

    pub fn get_plugin_from(&self, path: &str) -> Option<&Box<Plugin>> {
        let handle = match self.index.get(path) {
            Some(handle) => handle,
            None => return None
        };

        match handle {
            &Ok(ref handle) => Some(&handle.plugin),
            &Err(_) => None
        }
    }
}

pub struct RegistryIterator<'a> {
    iter: hash_map::Values<'a, String, PluginStatus>
}

impl <'a> RegistryIterator<'a> {
    pub fn new(registry: &'a Registry) -> RegistryIterator {
        RegistryIterator { iter: registry.index.values() }
    }
}

impl <'a> Iterator for RegistryIterator<'a> {
    type Item = &'a PluginStatus;

    fn next(&mut self) -> Option<&'a PluginStatus> {
        self.iter.next()
    }
}

impl fmt::Display for Registry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut table = table!(["Name", "Path", "Status"]);
        for path in self.index.keys() {
            let name = match self.lookup.get(path) {
                Some(name) => name,
                None => ""
            };

            // Fairly safe given we're iterating over keys.
            let handle = self.index.get(path).unwrap();
            if let &Ok(ref handle) = handle {
                table.add_row(Row::new(vec![
                    Cell::new(name), Cell::new(path), Cell::new("LOADED")
                ]));
            } else {
                table.add_row(Row::new(vec![
                    Cell::new(name), Cell::new(path), Cell::new("ERROR")
                ]));
            }
        }

        table.fmt(f)
    }
}

struct PluginConfig {
    root: toml::Value
}

impl PluginConfig {
    pub fn new(root: toml::Value) -> Self {
        PluginConfig { root: root }
    }
}

impl ConfigSource for PluginConfig {
    fn get_str(&self, path: &str) -> Option<&str> {
        match self.root.lookup(path) {
            Some(value) => value.as_str(),
            None => None
        }
    }

    fn get_int(&self, path: &str) -> Option<i64> {
        match self.root.lookup(path) {
            Some(value) => value.as_integer(),
            None => None
        }
    }

    fn get_float(&self, path: &str) -> Option<f64> {
        match self.root.lookup(path) {
            Some(value) => value.as_float(),
            None => None
        }
    }

    fn get_bool(&self, path: &str) -> Option<bool> {
        match self.root.lookup(path) {
            Some(value) => value.as_bool(),
            None => None
        }
    }

    fn get_str_slice(&self, path: &str) -> Option<&[&str]> {
        let slice = match self.root.lookup(path) {
            Some(value) => value.as_slice(),
            None => return None
        };

        println!("{:?}", slice);

        None
    }

    fn get_int_slice(&self, path: &str) -> Option<&[i64]> {
        None
    }

    fn get_float_slice(&self, path: &str) -> Option<&[f64]> {
        None
    }

    fn get_bool_slice(&self, path: &str) -> Option<&[bool]> {
        None
    }
}

pub struct Handle {
    pub name: String,
    pub version: Version,
    pub source: String,
    pub plugin: Box<Plugin>,
    pub library: Library
}
