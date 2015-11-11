extern crate irc;
extern crate hyper;
extern crate xml;

use irc::client::prelude::*;

use std::io::Read;

use hyper::Client;
use hyper::header::Connection;

fn main() {
	println!("Webscale scaling up...");

	let mut client = Client::new();

	let server = IrcServer::new("config.json").unwrap();
	server.identify().unwrap();

	for command in server.iter_cmd() {

		if let Ok(Command::PRIVMSG(chan, msg)) = command { // Ignore errors.
			if msg.contains("http:") {
				for x in msg.split(' ') {
					if(x.contains("http:"))
					{
						println!("Url received: {}", x);

						let mut res = client.get(x).header(Connection::close()).send().unwrap();

						let mut body = String::new();
						res.read_to_string(&mut body).unwrap();
					}
				}

				//server.send_privmsg(&chan, "Hi!").unwrap();
			}
		}

	}
}
