extern crate base64;
extern crate websocket;

use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::thread;
use websocket::sync::Server;
use websocket::OwnedMessage;

fn main() {
    // Bind websocket server
    let server = Server::bind("127.0.0.1:2795").unwrap();
    for request in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        thread::spawn(|| {
            if !request.protocols().contains(&"printerbridge".to_string()) {
                request.reject().unwrap();
                return;
            }

            let client = request.use_protocol("printerbridge").accept().unwrap();

            let (mut receiver, mut sender) = client.split().unwrap();

            for message in receiver.incoming_messages() {
                let message = message.unwrap();

                match message {
                    OwnedMessage::Close(_) => {
                        let message = OwnedMessage::Close(None);
                        sender.send_message(&message).unwrap();
                        return;
                    }
                    OwnedMessage::Ping(ping) => {
                        let message = OwnedMessage::Pong(ping);
                        sender.send_message(&message).unwrap();
                    }
                    OwnedMessage::Text(b64string) => {
                        let _str: &str = &b64string[1..b64string.len() - 1];
                        let _str = _str.replace("\n", "");
                        let _str = _str.replace("\r", "");
                        let _str = _str.replace("\r\n", "");
                        let _str = _str.replace("\\n", "");
                        match base64::decode(&_str) {
                            Ok(bytes) => {
                                let mut process = Command::new("lp")
                                    .stdin(Stdio::piped())
                                    .stdout(Stdio::piped())
                                    .spawn()
                                    .unwrap();
                                process.stdin.take().unwrap().write_all(&bytes).unwrap();
                            }
                            Err(err) => println!("Error! {}", err),
                        }
                    }
                    _ => sender.send_message(&message).unwrap(),
                }
            }
        });
    }
}
