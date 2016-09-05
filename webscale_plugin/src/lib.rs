//! Plugin extensions for the Webscale IRC bot.
//!
//! This library provides the client-facing API and a set of utility macros for 
//! developing extensions to the Webscale IRC bot. 

pub use std::collections::BTreeMap;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[macro_export]
macro_rules! export_plugin (
    ($x: expr, $c: ident) => ( 
        #[no_mangle]
        pub fn initialize_plugin(config: &PluginConfig) -> Box<Plugin> { 
            Box::new($c { }) 
        }

        #[no_mangle]
        pub static WS_PLUGIN_DESCRIPTION: $crate::PluginDescription = $crate::PluginDescription {
            name: $x,
            version: $crate::VERSION,
            initializer: b"initialize_plugin\0"
        };
    )
);

pub type PluginConfig = BTreeMap<String, String>;

pub trait Plugin: Send {
    fn on_connect(&self, &Link);
    fn on_disconnect(&self, &Link);
    fn on_message(&self, &Link, &str, &str);
    fn on_action(&self, &Link, &str, &str);
    fn on_join(&self, &Link, &str);
    fn on_leave(&self, &Link, &str);
}

#[derive(Debug)]
pub struct PluginDescription {
    pub name: &'static str,
    pub version: &'static str,
    pub initializer: &'static [u8]
}

pub struct User {
    pub name: String,
    pub login: String,
    pub hostname: String
}

pub trait Link {
    fn send(&self, target: &str, message: &str);
}
