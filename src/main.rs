extern crate irc;

use irc::client::prelude::*;

fn main() {
	println!("Webscale scaling up...");

	let server = IrcServer::new("config.json").unwrap();
	server.identify().unwrap();

	for message in server.iter() {
		println!("got a message");
	}
}
