extern crate irc;
extern crate hyper;
extern crate xml;
extern crate regex;

use irc::client::prelude::*;

use std::io::Read;
use regex::Regex;
use hyper::Client;
use hyper::header::Connection;

fn main() {
	println!("Webscale scaling up...");
    let config = Config {
        nickname: Some(format!("webscale")),
        alt_nicks: Some(vec![format!("webscale0x"), format!("webtech0")]),
        server: Some(format!("irc.freenode.net")),
        channels: Some(vec![format!("#0x")]),
        .. Default::default()
    };

    // Setup IRC server.
	let server = IrcServer::from_config(config).unwrap();
	server.identify().unwrap();

    // Setup HTTP client.
    let mut client = Client::new();

    // Common regex patterns.
    let url_pattern = Regex::new(r"(http[s]?://[^\s]+)").unwrap();
    let title_pattern = Regex::new(r"(?i)<title>(.+)</title>").unwrap();

	for message in server.iter() {
        let message = message.unwrap();
        if &message.command[..] == "PRIVMSG" {
            if let (Some(prefix), Some(msg)) = (message.prefix, message.suffix) {
                let user = prefix.split("!")
                    .collect::<Vec<&str>>()
                    .first()
                    .unwrap()
                    .to_string();

                if url_pattern.is_match(&msg) {
                    let url = url_pattern.captures(&msg).unwrap().at(0).unwrap();

                    let mut res = client.get(url).send().unwrap();
                    let mut body = String::new();
                    res.read_to_string(&mut body).unwrap();
                    
                    if title_pattern.is_match(&body) {
                        let title = title_pattern.captures(&body).unwrap().at(1).unwrap();
                        server.send_privmsg(&message.args[0], &vec!["[", &user, "] ", title].join(""));
                    }
                }
            }
        }
	}
}
