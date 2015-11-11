extern crate irc;

use irc::client::prelude::*;

fn main() {
	println!("Webscale scaling up...");

	let server = IrcServer::new("config.json").unwrap();
	server.identify().unwrap();

	for command in server.iter_cmd() {

		if let Ok(Command::PRIVMSG(chan, msg)) = command { // Ignore errors.
			if msg.contains("http:") {
				for x in msg.split(' ') {
					println!("{}", x);
				}

				//server.send_privmsg(&chan, "Hi!").unwrap();
			}
		}

	}
}
