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


// MessageHandler is a simple abstraction for different features
//
// Right now the handling is limited string input and output.
// To not react to the message received, return None.
//
// If message is received, the reply is send to the same source where it was originating
//
// All messages are currently send to all handlers.
// 
// TODO: Information of the source should be passed to the handler
trait MessageHandler {

    // Get an message, if doing something with it, send reply back (reply goes always back to 
    fn handle_message(&mut self, message :&str) -> Option<String>;
}

struct TitleScrapper;

impl MessageHandler for TitleScrapper {
    fn handle_message(&mut self, message :&str) -> Option<String> {

        // Move to the struct or something
        let url_pattern = Regex::new(r"(http[s]?://[^\s]+)").unwrap();

        if url_pattern.is_match(&message) {
            let url = url_pattern.captures(&message).unwrap().at(0).unwrap();

            println!("We should fetch url: {}", url);

            match get_title_for_url(url) {
                Ok(title) => {
                    return Some(vec!["Title: ", &title].join(""));
                } ,
                Err(err) => println!("Title fetch failed: {}", err),
            };
        }

        None
    }
}

// Dummy example message handler
struct Pinger {
    // How many pongs are we going to give out 
    // Limited amount of pongs per boot
    count: i8
}

impl MessageHandler for Pinger {
    fn handle_message(&mut self, message :&str) -> Option<String> {
        if self.count <= 0 {
            return None;
        }

        if message.contains("!ping") {
            self.count = self.count - 1;

            return Some(String::from("pong"));
        }

        None
    }
}

fn main() {
	println!("Webscale is scaling up...");

    // Setup IRC server.
	let server = IrcServer::new("webscale.json").unwrap();
	server.identify().unwrap();

    // Contains all the different message handlers
    let mut message_handlers: Vec<Box<MessageHandler>> = Vec::new();

    // Add all different handlers into use
    message_handlers.push(Box::new(TitleScrapper {}));
    message_handlers.push(Box::new(Pinger {count : 5}));

	for message in server.iter() {
		let message = message.unwrap(); //If IRC message doesn't unwrap, we probably lost connection

		print!("{}", message);

		match message.command {
			Command::PRIVMSG(ref target, ref msg) => {

                for handler in message_handlers.iter_mut() {
                    match handler.handle_message(msg) {
                        Some(msg) => {
                            server.send_privmsg(target, &msg);
                        },
                        None => (),
                    }
                }

			},
			_ => (),

		}

	}

    println!("Lost connection, shutting down...");
}
