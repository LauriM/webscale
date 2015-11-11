extern crate irc;
extern crate curl;

use irc::client::prelude::*;
use curl::http;

fn main() {
	println!("Webscale scaling up...");

	let server = IrcServer::new("config.json").unwrap();
	server.identify().unwrap();

	for command in server.iter_cmd() {

		if let Ok(Command::PRIVMSG(chan, msg)) = command { // Ignore errors.
			if msg.contains("http:") {
				for x in msg.split(' ') {
					if(x.contains("http:"))
					{
						println!("Url received: {}", x);

						let resp = http::handle().get(x).exec().unwrap();

						//TODO: bug, failing on redirections
						if(resp.get_code() == 200)
						{
							println!("Content: {:?}", resp.get_body());
						}
						else
						{
							println!("invalid url");
						}
					}
				}

				//server.send_privmsg(&chan, "Hi!").unwrap();
			}
		}

	}
}
