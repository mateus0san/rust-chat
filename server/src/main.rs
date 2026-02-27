use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use sbmp::read::FrameReader;
use sbmp::{ContentType, Frame, SBMPError};
use server::{Client, ConnectionEnd, Message, ThreadPool};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1337").expect("ERROR: could not start the server");
    let (sender, receiver) = mpsc::channel::<Message>();
    let pool = ThreadPool::build(2);

    thread::spawn(|| server(receiver));
    eprintln!("Serving on 0.0.0.0:1337");

    for stream in listener.incoming() {
        let sender = sender.clone();

        match stream {
            Err(e) => eprintln!("INFO: new stream returned an error {e}"),
            Ok(stream) => {
                pool.execute(|| {
                    client(stream, sender);
                });
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

fn new_client(clients: &mut HashMap<SocketAddr, Client>, mut client: Client) {
    let ip = client.ip();

    if clients.get(&ip).is_some() {
        let err = "INFO: This username is already on the server";
        let _ = client.write(err);
    } else {
        clients.insert(client.ip(), client);
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
    let mut reader = FrameReader::new(reader);

    let Some(client) = login(stream, &mut reader) else {
        return Ok(ConnectionEnd::Normal);
    };
    let username = client.username().to_string();
    let ip = client.ip();

    if sender.send(Message::NewClient(client)).is_err() {
        return Ok(ConnectionEnd::ReceiverDropped);
    };

    let result = loop {
        let Some(frame) = read(&mut reader, ip) else {
            break Ok(ConnectionEnd::Normal);
        };

        let Some(client_msg) = get_message(frame, &username) else {
            continue;
        };

        if sender.send(Message::Broadcast(client_msg)).is_err() {
            break Ok(ConnectionEnd::ReceiverDropped);
        }
    };

    let _ = sender.send(Message::Drop(ip));
    result
}

fn get_message(frame: Frame, username: &str) -> Option<String> {
    if frame.get_header().content_type() != ContentType::UTF8 {
        return None;
    }

    if let Ok(msg) = String::from_utf8(frame.get_payload()) {
        let msg = msg.trim();
        if msg.is_empty() {
            return None;
        }
        return Some(format!("[{username}]:{msg}"));
    }

    None
}

fn read(reader: &mut FrameReader<TcpStream>, ip: SocketAddr) -> Option<Frame> {
    let frame = match reader.read_frame() {
        Ok(frame) => frame,
        Err(e) => {
            eprintln!("ERROR: read_messages from {ip}: {:#?}", e);
            return None;
        }
    };

    Some(frame)
}

fn login(stream: TcpStream, reader: &mut FrameReader<TcpStream>) -> Option<Client> {
    let mut client = match Client::try_new(stream) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("ERROR: login 'Client::try_new': {:#?}", e);
            return None;
        }
    };

    if let Err(e) = client.write("Type your username: ") {
        eprintln!("ERROR: login 'client.write': {:#?}", e);
        return None;
    }

    let frame = match reader.read_frame() {
        Ok(frame) => frame,
        Err(e) => {
            eprintln!("ERROR: login 'reader.read_frame': {:#?}", e);
            return None;
        }
    };

    if frame.get_header().content_type() != ContentType::UTF8 {
        let err = "ERROR: login: content type should be UTF8";
        let _ = client.write(err);
        eprintln!("{err}");
        return None;
    }

    let username = match String::from_utf8(frame.get_payload()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: login 'from_utf8' {:#?}", e);
            let _ = client.write("Username invalid: bad encoding");
            return None;
        }
    };

    if client.set_username(username).is_none() {
        let _ = client.write("Username should have less than 32 chars");
        return None;
    };

    Some(client)
}
