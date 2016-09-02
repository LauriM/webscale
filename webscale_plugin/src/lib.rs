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

pub trait Plugin {
    fn on_connect(&self);
    fn on_disconnect(&self);
    fn on_message(&self);
}

#[derive(Debug)]
pub struct PluginDescription {
    pub name: &'static str,
    pub version: &'static str,
    pub initializer: &'static [u8]
}
