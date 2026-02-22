use std::{
    collections::{HashMap, hash_map::Entry},
    io::{self, ErrorKind},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use chat::{Client, ConnectionEnd, Message, Reader};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1337").expect("ERROR: could not start the server");
    let (sender, receiver) = mpsc::channel::<Message>();

    eprintln!("Serving on 0.0.0.0:1337");
    thread::spawn(|| server(receiver));

    for stream in listener.incoming() {
        let sender = sender.clone();

        match stream {
            Err(e) => eprintln!("INFO: new stream returned an error {e}"),
            Ok(stream) => {
                thread::spawn(|| match handle_connection(stream, sender) {
                    Err(e) => eprintln!("INFO: connection failed: {e}"),
                    Ok(ConnectionEnd::ReceiverDropped) => {
                        eprintln!("ERROR: Receiver Dropped, it should not happen")
                    }
                    _ => (),
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
            Message::NewClient(client) => {
                if let Some(new_client) = new_client(&mut clients, client) {
                    eprintln!("INFO: New client {}", new_client.ip());
                } else {
                    eprintln!("INFO: Ip address of new client is already on the server.")
                }
            }
        }
    }
}

fn new_client(clients: &mut HashMap<SocketAddr, Client>, client: Client) -> Option<&mut Client> {
    match clients.entry(client.ip()) {
        Entry::Occupied(_) => None,
        Entry::Vacant(e) => Some(e.insert(client)),
    }
}

fn new_message(msg: String, clients: &mut HashMap<SocketAddr, Client>) {
    let _removed_clients: HashMap<SocketAddr, Client> =
        clients.extract_if(|_k, v| v.write(&msg).is_err()).collect();
}

fn handle_connection(
    stream: TcpStream,
    sender: mpsc::Sender<Message>,
) -> Result<ConnectionEnd, io::Error> {
    let reader = stream.try_clone()?;

    let client = Client::try_new(stream)?;
    let ip = client.ip();

    if sender.send(Message::NewClient(client)).is_err() {
        return Ok(ConnectionEnd::ReceiverDropped);
    };

    let mut buffer = Reader::new(reader, 120);

    let result = loop {
        let msg = match buffer.read_line() {
            Ok(msg) => msg,
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                break Ok(ConnectionEnd::Normal);
            }
            Err(e) => break Err(e),
        };

        eprintln!("INFO: [{ip}] sent a new message {msg}");
        if sender.send(Message::Broadcast(msg)).is_err() {
            break Ok(ConnectionEnd::ReceiverDropped);
        }

        // timeout to send the next message
        thread::sleep(Duration::from_secs(1));
    };

    let _ = sender.send(Message::Drop(ip));
    result
}
