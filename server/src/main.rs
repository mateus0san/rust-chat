use std::{
    collections::{HashMap, hash_map::Entry},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use sbmp::SBMPError;
use sbmp::read::FrameReader;
use server::{Client, ConnectionEnd, Message};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1337").expect("ERROR: could not start the server");
    let (sender, receiver) = mpsc::channel::<Message>();

    thread::spawn(|| server(receiver));
    eprintln!("Serving on 0.0.0.0:1337");

    for stream in listener.incoming() {
        let sender = sender.clone();

        match stream {
            Err(e) => eprintln!("INFO: new stream returned an error {e}"),
            Ok(stream) => {
                thread::spawn(|| client(stream, sender));
            }
        }
    }
}

fn server(receiver: Receiver<Message>) {
    let mut clients = HashMap::new();

    for msg in receiver {
        match msg {
            Message::Broadcast(msg) => new_message(msg, &mut clients),
            Message::Drop(ip) => {
                clients.remove(&ip);
            }
            Message::NewClient(client) => new_client(&mut clients, client),
        }
    }
}

fn new_message(msg: String, clients: &mut HashMap<SocketAddr, Client>) {
    let _removed_clients: HashMap<SocketAddr, Client> =
        clients.extract_if(|_k, v| v.write(&msg).is_err()).collect();
}

fn new_client(clients: &mut HashMap<SocketAddr, Client>, client: Client) {
    match clients.entry(client.ip()) {
        Entry::Occupied(_) => {
            eprintln!("INFO: Ip address of new client is already on the server.");
        }
        Entry::Vacant(e) => {
            eprintln!("INFO: New client {}", client.ip());
            e.insert(client);
        }
    }
}

fn client(stream: TcpStream, sender: Sender<Message>) {
    match handle_connection(stream, sender) {
        Err(e) => eprintln!("INFO: connection failed: {e:#?}"),
        Ok(ConnectionEnd::ReceiverDropped) => {
            eprintln!("ERROR: Receiver Dropped, it should not happen")
        }
        _ => (),
    }
}

fn handle_connection(
    stream: TcpStream,
    sender: mpsc::Sender<Message>,
) -> Result<ConnectionEnd, SBMPError> {
    let reader = stream.try_clone()?;

    let client = Client::try_new(stream)?;
    let ip = client.ip();

    if sender.send(Message::NewClient(client)).is_err() {
        return Ok(ConnectionEnd::ReceiverDropped);
    };

    let mut buffer = FrameReader::new(reader);

    let result = loop {
        let frame = match buffer.read_frame() {
            Ok(s) => s,
            Err(e) => break Err(e),
        };

        let client_message = String::from_utf8(frame.get_payload()).expect("remove this later");
        if sender.send(Message::Broadcast(client_message)).is_err() {
            break Ok(ConnectionEnd::ReceiverDropped);
        }
    };

    let _ = sender.send(Message::Drop(ip));
    result
}
