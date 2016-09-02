#[macro_use]
extern crate webscale_plugin;

use webscale_plugin::{Plugin, PluginConfig};

pub struct TestPlugin { }

impl Plugin for TestPlugin {
    fn on_connect(&self) { 
        println!("HELLO I AM PLUGIN.");
    }

    fn on_disconnect(&self) { 

    }

    fn on_message(&self) { 

    }
}

export_plugin!("my-plugin", TestPlugin);
