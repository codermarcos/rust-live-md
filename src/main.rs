extern crate websocket;

use pulldown_cmark::{html, Options, Parser};
use std::thread;
use websocket::sync::Server;
use websocket::OwnedMessage;

fn parser(text: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&text, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    let replacer = gh_emoji::Replacer::new();
    html_output = replacer.replace_all(&html_output).to_string();

    html_output
}

fn main() {
    let server = Server::bind("127.0.0.1:2794").unwrap();

    for request in server.filter_map(Result::ok) {
        thread::spawn(|| {
            if !request.protocols().contains(&"live-md".to_string()) {
                request.reject().unwrap();
                return;
            }

            let mut client = request.use_protocol("live-md").accept().unwrap();

            let ip = client.peer_addr().unwrap();

            println!("Connection from {}", ip);

            let initial = parser(
                "# Teste do titulo\n\
                Meu nome é Marcos eu consigo add emoji :poop:\n\
                * Isso vai ser uma lista\n\
                * Isso é uma prop em **negrito**\n\
                * Isso é uma prop em *italico*\n\
                * Isso é um [link](TESTANDO LINK)\n\
                ## Teste de sub titulo\n\
                ### Teste de subtitle\n\
                #### Teste de subtitle",
            );

            let message = OwnedMessage::Text(initial.to_string());
            client.send_message(&message).unwrap();

            let (mut receiver, mut sender) = client.split().unwrap();

            for message in receiver.incoming_messages() {
                let message = message.unwrap();

                match message {
                    OwnedMessage::Close(_) => {
                        let message = OwnedMessage::Close(None);
                        sender.send_message(&message).unwrap();
                        println!("Client {} disconnected", ip);
                        return;
                    }
                    OwnedMessage::Ping(ping) => {
                        let message = OwnedMessage::Pong(ping);
                        sender.send_message(&message).unwrap();
                    }
                    OwnedMessage::Text(text) => {
                        let html_output = parser(&text);
                        let message = OwnedMessage::Text(html_output);

                        sender.send_message(&message).unwrap();
                    }
                    _ => sender.send_message(&message).unwrap(),
                }
            }
        });
    }
}
