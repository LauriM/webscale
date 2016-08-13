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

fn get_title_for_url(url :&str) -> Result<String, String> {
    let mut client = Client::new();
    client.set_read_timeout(Some(Duration::new(5, 0)));

    let mut body = match client.get(url).send() {
        Ok(mut res) => {
            let mut body = String::new();

            // 15k is a nice round number to keep the trolls away
            res.take(15000).read_to_string(&mut body);

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

    let mut title: String = body[start_pos..end_pos].to_owned();

    // Trim whitespace out of titles because webdevs are just amazing...
    title = match title.trim().parse()
    {
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

struct PatternData {
    pattern: String,
    reply: String
}

impl PatternData {
    fn new(pattern: String, reply: String) -> PatternData {
        PatternData { pattern:pattern, reply:reply }
    }
}


//Replies to certain pattern of messages with predefined answers.
//Useful to provide links to certain resources, etc.
//
//Answers are stored in a file on disk.
struct Replier {
    patterns: Vec<PatternData>
}

impl Replier {

    fn loadPatterns(&mut self) {
        let mut file = match File::open("patterns.txt") {
            Err(e) => {
                println!("Could not find patterns.txt, not using pattern replies");
                return 
            },
            Ok(file) => file
        };

        let mut reader = BufReader::new(file);

        let mut line = String::new();

        while reader.read_line(&mut line).unwrap() > 0 {

            {
                let split: Vec<&str> = line.split("|").collect();

                println!("{}", line);
                println!("{}", split[0]);

                let pattern: String = String::from(split[0]);
                let reply: String = String::from(split[1]);

                self.patterns.push(PatternData {pattern: pattern, reply: reply} );
            }

            line.clear();
        }
    }

}

impl MessageHandler for Replier {
    fn handle_message(&mut self, message :&str) -> Option<String> {

        for p in self.patterns.iter_mut() {
            if message.contains(&p.pattern) {
                return Some(p.reply.to_owned());
            }
        }
        None
    }
}

struct Updater {
}

impl MessageHandler for Updater {
    fn handle_message(&mut self, message :&str) -> Option<String> {
        if message.contains("!rebuild") {
            panic!("herp");
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

    let mut replier: Replier = Replier { patterns: Vec::new() };

    replier.loadPatterns();

    // Add all different handlers into use
    message_handlers.push(Box::new(TitleScrapper {}));
    message_handlers.push(Box::new(Updater {}));
    message_handlers.push(Box::new(replier));

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
