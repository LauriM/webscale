extern crate hyper;
extern crate irc;
extern crate xml;
extern crate regex;
extern crate config;
extern crate url;

use std::path::Path;
use irc::client::prelude::*;
use std::io::Read;
use regex::Regex;
use hyper::Client;
use hyper::header::Connection;
use config::reader;
use config::types::Value;
use config::types::ScalarValue;
use hyper::Url;
use hyper::client::Request;
use url::ParseError;

fn main() {
	println!("Webscale scaling up...");

	println!("Reading configs...");

	let confFile = reader::from_file(Path::new("webscale.conf")).unwrap();

	/*
	if !confFile.is_ok()
	{
		println!("webscale.conf missing! Please check the package for example configuration.");
	}
	*/

//	let configuration = confFile.unwrap();

	let nickname = confFile.lookup_str("webscale.nickname");
	let altnick = confFile.lookup_str("webscale.altnick");
	let server = confFile.lookup_str("webscale.server");
	let channel = confFile.lookup_str("webscale.channel");

    let config = Config {
        nickname: Some(String::from(nickname.unwrap())),
        alt_nicks: Some(vec![ String::from(altnick.unwrap()), format!("wartech0r") ]),
        server: Some(String::from(server.unwrap())),
        channels: Some(vec![ String::from(channel.unwrap()) ]),
        .. Default::default()
    };

    // Setup IRC server.
	let server = IrcServer::from_config(config).unwrap();
	server.identify().unwrap();

    // Setup HTTP client.
    let client = Client::new();

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
					// Crash probably somewhere around here, not handling error correctly
                    let url = url_pattern.captures(&msg).unwrap().at(0).unwrap();

					let mut res = client.get(url).send().unwrap();
					assert_eq!(res.status, hyper::Ok);

					// check that its ok
					if res.status == hyper::Ok {

						let mut body = String::new();
						res.read_to_string(&mut body).unwrap();

						if title_pattern.is_match(&body) {
							let title = title_pattern.captures(&body).unwrap().at(1).unwrap();
							server.send_privmsg(&message.args[0], &vec!["Title: ", title].join(""));
						}
					}

                }
            }
        }
	}
}
