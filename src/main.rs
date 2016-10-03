extern crate ws;

use ws::{listen, Handler, Sender, Handshake, Result, Message, CloseCode};

#[macro_use]
extern crate log;
extern crate log4rs;

struct Server {
    out: Sender,
}

impl Handler for Server {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        // Echo the message back
        self.out.send(msg)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        // The WebSocket protocol allows for a utf8 reason for the closing state after the
        // close code. WS-RS will attempt to interpret this data as a utf8 description of the
        // reason for closing the connection. I many cases, `reason` will be an empty string.
        // So, you may not normally want to display `reason` to the user,
        // but let's assume that we know that `reason` is human-readable.
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => println!("The client is leaving the site."),
            _ => println!("The client encountered an error: {}", reason),
        }
    }
}

fn main() {
    let logging_filename = "logging.yml";
    log4rs::init_file(logging_filename, Default::default()).unwrap();
    info!("Starting Eden");
    // Now, instead of a closure, the Factory returns a new instance of our Handler.
    listen("127.0.0.1:3012", |out| Server { out: out }).unwrap()
}
