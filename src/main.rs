use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc,
    thread,
};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1337").expect("ERROR: could not start the server");
    let (sender, receiver) = mpsc::channel::<Message>();

    thread::spawn(|| {
        let mut clients = HashMap::new();
        for msg in receiver {
            match msg {
                Message::Broadcast(msg) => new_message(msg, &mut clients),
                Message::NewClient(client) => {
                    if let Some(new_client) = new_client(&mut clients, client) {
                        eprintln!("INFO: New client {}", new_client.ip);
                    } else {
                        eprintln!("INFO: Ip address of new client is already on the server.")
                    }
                }
            }
        }
    });

    for stream in listener.incoming() {
        let sender = sender.clone();
        match stream {
            Err(e) => eprintln!("INFO: new stream returned an error {e}"),
            Ok(stream) => {
                thread::spawn(|| match handle_connection(stream, sender) {
                    Err(e) => eprintln!("INFO: connection failed: {e}"),
                    Ok(e) => eprintln!("INFO: {e}"),
                });
            }
        }
    }
}

fn new_client(clients: &mut HashMap<SocketAddr, Client>, client: Client) -> Option<&mut Client> {
    if clients.get(&client.ip).is_some() {
        client
            .writer
            .shutdown(std::net::Shutdown::Both)
            .expect("This stream must to shutdown");
        return None;
    }

    Some(clients.entry(client.ip).or_insert(client))
}

fn new_message(msg: String, clients: &mut HashMap<SocketAddr, Client>) {
    let msg = msg.as_bytes();
    let _removed_clients: HashMap<SocketAddr, Client> = clients
        .extract_if(|_k, v| v.writer.write_all(msg).is_err())
        .collect();
}

enum Message {
    Broadcast(String),
    NewClient(Client),
}

struct Client {
    ip: SocketAddr,
    writer: TcpStream,
}

impl Client {
    fn try_new(stream: TcpStream) -> Result<Client, io::Error> {
        let ip = stream.peer_addr()?;

        Ok(Client::new(stream, ip))
    }

    fn new(writer: TcpStream, ip: SocketAddr) -> Self {
        Self { ip, writer }
    }
}

fn handle_connection(
    stream: TcpStream,
    sender: mpsc::Sender<Message>,
) -> Result<&'static str, io::Error> {
    let recv_msg = "receiver dropped";
    let end_normal = "connection closed normally";

    let reader = stream.try_clone()?;

    let client = Client::try_new(stream)?;
    if sender.send(Message::NewClient(client)).is_err() {
        return Ok(recv_msg);
    }

    let mut reader = BufReader::new(reader);

    loop {
        let mut msg = String::with_capacity(120);

        match reader.by_ref().take(120).read_line(&mut msg) {
            Ok(0) => break Ok(end_normal),
            Err(e) => break Err(e),
            _ => {
                if sender.send(Message::Broadcast(msg)).is_err() {
                    break Ok(recv_msg);
                }
            }
        }
    }
}
