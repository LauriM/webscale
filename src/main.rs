extern crate hyper;
extern crate irc;
extern crate regex;

use irc::client::prelude::*;
use hyper::Client;
use regex::Regex;
use std::time::Duration;
use std::io::BufReader;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

fn get_title_for_url(url: &str) -> Result<String, String> {
    let mut client = Client::new();
    client.set_read_timeout(Some(Duration::new(5, 0)));

    let body = match client.get(url).send() {
        Ok(res) => {
            let mut body = String::new();

            // 15k is a nice round number to keep the trolls away
            res.take(15000).read_to_string(&mut body);

            body
        }
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

    let mut title: String = body[start_pos..end_pos].to_owned();

    // Trim whitespace out of titles because webdevs are just amazing...
    title = match title.trim().parse() {
        Ok(title) => title,
        Err(_) => return Err(String::from("failed to trim the title!")),
    };

    title = title.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&x27;", "\\")
        .replace("&lt;", "<")
        .replace("&#39;", "'")
        .replace("&#039;", "'")
        .replace("&gt;", ">");

    Ok(title)
}

// Contains relevant information to a single message
struct IrcMessage {
    target :String, // privmsg target
    message :String, // Message itself
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

    // message, what is send from IRC server
    // tx, channel that can be used to send replies back to the IRC server
    fn handle_message(&mut self, message :IrcMessage, tx :mpsc::Sender<IrcMessage>);
}

struct TitleScrapper;

impl MessageHandler for TitleScrapper {
    fn handle_message(&mut self, message :IrcMessage, tx :mpsc::Sender<IrcMessage>) {

        // Move to the struct or something
        let url_pattern = Regex::new(r"(http[s]?://[^\s]+)").unwrap();

        if url_pattern.is_match(&message.message) {

            // Thread spawned to make sure invalid links won't kill the whole bot
            thread::spawn(move || {
                let target = message.target;
                let message = message.message;

                let url = url_pattern.captures(&message).unwrap().at(0).unwrap();

                println!("Fetching URL: {}", url);

                match get_title_for_url(url) {
                    Ok(title) => {
                        let reply = IrcMessage { target: target, message: title };
                        tx.send(reply);
                    } ,
                    Err(err) => println!("Failed to fetch title for: {}", err),
                };
            });
        }
    }
}

struct PatternData {
    pattern: String,
    reply: String,
}

// Replies to certain pattern of messages with predefined answers.
// Useful to provide links to certain resources, etc.
//
// Answers are stored in a file on disk.
struct Replier {
    patterns: Vec<PatternData>,
}

impl Replier {
    fn load_patterns(&mut self) {
        //TODO: Hazard unwrap, fix
        let mut file = match File::open("patterns.txt") {
            Err(e) => {
                println!("Could not find patterns.txt, not using pattern replies");
                return;
            }
            Ok(file) => file,
        };

        let mut reader = BufReader::new(file);

        let mut line = String::new();

        while reader.read_line(&mut line).unwrap() > 0 {

            {
                let split: Vec<&str> = line.split("|").collect();

                let pattern: String = String::from(split[0]);
                let reply: String = String::from(split[1]);

                self.patterns.push(PatternData {
                    pattern: pattern,
                    reply: reply,
                });
            }

            line.clear();
        }
    }
}

impl MessageHandler for Replier {
    fn handle_message(&mut self, message :IrcMessage, tx :mpsc::Sender<IrcMessage>) {
        for p in self.patterns.iter_mut() {
            if message.message.contains(&p.pattern) {

                let reply = IrcMessage { target: message.target.to_owned(), message: p.reply.to_owned() };
                tx.send(reply);
            }
        }
    }
}

struct Updater {
}

impl MessageHandler for Updater {
    fn handle_message(&mut self, message :IrcMessage, tx :mpsc::Sender<IrcMessage>) {
        if message.message.contains("!rebuild") {
            panic!("herp"); // we just crash the whole damn thing
            //TODO: Add proper exit handling
        }
    }
}

fn main() {
    println!("Webscale is scaling up...");

    // -- Setup IRC server.
    let server = IrcServer::new("webscale.json").unwrap();
    server.identify().unwrap();

    // -- Setup handlers
    let mut replier: Replier = Replier { patterns: Vec::new() };

    replier.load_patterns();

    // -- List message handlers
    let mut message_handlers: Vec<Box<MessageHandler>> = Vec::new();

    // Add all different handlers into use
    message_handlers.push(Box::new(TitleScrapper {}));
    message_handlers.push(Box::new(Updater {}));
    message_handlers.push(Box::new(replier));


    // Thread handling stuff send to the server
    let server_outbound = server.clone();
    let (server_outbound_tx, server_outbound_rx): (Sender<IrcMessage>, Receiver<IrcMessage>) = mpsc::channel();

    thread::spawn(move || {
        loop {
            let msg = server_outbound_rx.recv().unwrap();
            server_outbound.send_privmsg(&msg.target, &msg.message);
        }
    });

    // Thread handling the irc connection
    for message in server.iter() {
        let message = message.unwrap(); // TODO: handle this with more care

        match message.command {
            Command::PRIVMSG(ref target, ref msg) => {
                for handler in message_handlers.iter_mut() {
                    let irc_msg = IrcMessage { target: target.to_owned(), message: msg.to_owned() };
                    handler.handle_message(irc_msg, server_outbound_tx.clone());
                }
            }
            _ => (),
        }

        // "Logging" to stdout
        print!("{}", message);
    }

    println!("Lost connection, shutting down...");
}
