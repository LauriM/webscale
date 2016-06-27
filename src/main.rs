extern crate hyper;
extern crate irc;
extern crate config;
extern crate regex;
extern crate rquery;

use std::io::Read;
use irc::client::prelude::*;
use std::path::Path;
use config::reader;
use hyper::Client;
use regex::Regex;
use std::result;

fn get_title_for_url(url :&str) -> Result<String, String> {
    let client = Client::new();

    let mut body = match client.get(url).send() {
        Ok(mut res) => {
            let mut body = String::new();
            res.read_to_string(&mut body).unwrap();

            body
        },
        Err(err) => return Err(err.to_string()),
    };

    // Finding the title from the body
    
    let start_pos = match body.find("<title>") {
        Some(res) => res + 7,
        None => return Err(String::from("Title missing")),
    };

    let end_pos = match body.find("</title>") {
        Some(res) => res,
        None => return Err(String::from("Title missing")),
    };

    //TODO: some funny "</" characthers left sometimes in the title
    let title: String = body.chars().skip(start_pos).take(end_pos - start_pos).collect();

    Ok(title)
}

fn main() {
	println!("Webscale scaling up...");

	println!("Reading configs...");

	// Configuration
	//TODO: Configuration could be much more flexible
	let conf_file = reader::from_file(Path::new("webscale.conf")).unwrap();

	let nickname = conf_file.lookup_str("webscale.nickname");
	let altnick = conf_file.lookup_str("webscale.altnick");
	let server = conf_file.lookup_str("webscale.server");
	let channel = conf_file.lookup_str("webscale.channel");

    let config = Config {
        nickname: Some(String::from(nickname.unwrap())),
        alt_nicks: Some(vec![ String::from(altnick.unwrap()), format!("wartech0r") ]),
        server: Some(String::from(server.unwrap())),
        channels: Some(vec![ String::from(channel.unwrap()) ]),
        .. Default::default()
    };

    // Setup IRC server.
	let server = IrcServer::from_config(config).unwrap();
	server.identify();

    // Used to catch the url's from incoming messages
    let url_pattern = Regex::new(r"(http[s]?://[^\s]+)").unwrap();

	for message in server.iter() {
		let message = message.unwrap(); //If IRC message doesn't unwrap, we probably lost connection

		print!("{}", message);

		match message.command {
			Command::PRIVMSG(ref target, ref msg) => {

                if msg.contains("!ping") {
                    server.send_privmsg(target, "pong");
                }

                if url_pattern.is_match(&msg) {
                    let url = url_pattern.captures(&msg).unwrap().at(0).unwrap();

                    println!("We should fetch url: {}", url);

                    match get_title_for_url(url) {
                        Ok(title) => {
                            server.send_privmsg(target, &vec!["Title: ", &title].join(""));
                        } ,
                        Err(err) => println!("Title fetch failed: {}", err),
                    };
                }

			},
			_ => (),
		}

	}

    println!("Lost connection, shutting down...");
}
