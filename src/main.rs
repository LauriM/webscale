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
            res.read_to_string(&mut body);

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

    let title: String = body[start_pos..end_pos].to_owned();

    Ok(title)
}

fn main() {
	println!("Webscale is scaling up...");

    // Setup IRC server.
	let server = IrcServer::new("webscale.json").unwrap();
	server.identify().unwrap();

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
