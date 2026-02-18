use std::{
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    let listener = TcpListener::bind(server_address()).expect("ERROR: Failed to bind server");
    let server_address = listener
        .local_addr()
        .expect("ERROR: Failed to find server address");

    println!("Serving on {server_address}");

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ERROR: {e}");
                continue;
            }
        };
        thread::spawn(move || {
            handle_client(stream);
        });
    }
}

fn server_address() -> String {
    std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:1337".to_string())
}

struct Client {
    username: String,
    writer: TcpStream,
    reader: BufReader<TcpStream>,
    ip: String,
}

#[derive(Debug)]
enum ClientError {
    Io(io::Error),
    Eof,
}

impl Client {
    fn try_new(mut stream: TcpStream) -> Result<Self, ClientError> {
        let ip = stream.peer_addr().map_err(ClientError::Io)?.to_string();

        stream
            .write_all("Type your username (max 16 characters): ".as_bytes())
            .map_err(ClientError::Io)?;

        const MAX_USERNAME_LEN: usize = 16;
        let mut username = String::with_capacity(MAX_USERNAME_LEN);

        let reader = stream.try_clone().map_err(ClientError::Io)?;
        let mut buffer = BufReader::new(reader);
        match buffer.read_line(&mut username) {
            Ok(0) => Err(ClientError::Eof),
            Err(e) => Err(ClientError::Io(e)),
            Ok(_) => Ok(Client::new(username, stream, buffer, ip)),
        }
    }

    fn new(username: String, writer: TcpStream, reader: BufReader<TcpStream>, ip: String) -> Self {
        Self {
            username,
            writer,
            reader,
            ip,
        }
    }
}

fn handle_client(stream: TcpStream) {
    let client = match Client::try_new(stream) {
        Ok(client) => client,
        Err(ClientError::Io(e)) => {
            eprintln!("ERROR: client_handle std::io::Error {e}");
            return;
        }
        Err(e) => {
            eprint!("ERROR: client_handle");
            dbg!(e);
            return;
        }
    };

    eprintln!(
        "New client {} at the address {}",
        client.username, client.ip
    );

    // const MAX_BYTES: usize = 120;
    // let mut message = String::with_capacity(MAX_BYTES);

    // loop {
    //     match buffer.read_line(&mut message) {
    //         Ok(0) => {
    //             eprintln!("INFO: {} buffer returned EOF", client.ip);
    //             break;
    //         }
    //         Ok(b) => eprintln!(
    //             "INFO: Number of bytes read from {}: {b}:{}",
    //             client.ip,
    //             message.trim()
    //         ),
    //         Err(err) => {
    //             eprintln!("ERROR: {err}");
    //             break;
    //         }
    //     }

    //     message.clear();
    // }
}
