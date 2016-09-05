#[macro_use]
extern crate webscale_plugin;

use webscale_plugin::*;

pub struct TestPlugin { }

impl Plugin for TestPlugin {
    fn on_connect(&self, server: &Link) { 
        println!("Connected to server.");
    }

    fn on_disconnect(&self, server: &Link) { 
        println!("Disconnected from server.");
    }

    fn on_message(&self, server: &Link, target: &str, message: &str) { 
        server.send(target, "What did you say to me?");
    }

    fn on_action(&self, server: &Link, target: &str, action: &str) { 

    }

    fn on_join(&self, server: &Link, channel: &str) { 

    }

    fn on_leave(&self, server: &Link, channel: &str) { 

    }
}

export_plugin!("my-plugin", TestPlugin);
