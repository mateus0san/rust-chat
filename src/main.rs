use std::{
    io::Write,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

struct Server {
    listener: TcpListener,
    addr: SocketAddr,
    sender: mpsc::Sender<chat::Client>,
    connection: Arc<Mutex<Connection>>,
}

enum Connection {
    Drop,
    Accept,
    End,
}

impl Connection {
    fn drop(mut stream: TcpStream) {
        let _ = stream.write_all("The server is full".as_bytes());
        drop(stream);
        eprintln!("INFO: Dropping connection");
        thread::sleep(Duration::from_secs(5));
    }
}

impl Server {
    fn new(listener: TcpListener, addr: SocketAddr, sender: mpsc::Sender<chat::Client>) -> Self {
        Self {
            listener,
            addr,
            sender,
            connection: Arc::new(Mutex::new(Connection::Accept)),
        }
    }

    fn bind_server() -> (Self, mpsc::Receiver<chat::Client>) {
        let addr = "0.0.0.0:1337";
        eprintln!("Binding server at {addr}");

        let listener = TcpListener::bind(addr).expect("ERROR: Failed to bind server");
        let server_address = listener
            .local_addr()
            .expect("ERROR: Failed to get server address");

        let (sender, receiver) = mpsc::channel();

        (Server::new(listener, server_address, sender), receiver)
    }

    fn run(&self) {
        eprintln!("Serving on {}", self.addr);

        for stream in self.listener.incoming() {
            let Ok(stream) = stream else {
                let _ = dbg!(stream);
                continue;
            };

            let connection = self.connection.lock().unwrap();
            match *connection {
                Connection::Accept => (),
                Connection::End => return,
                Connection::Drop => {
                    Connection::drop(stream);
                    continue;
                }
            }

            let connection = Arc::clone(&self.connection);
            let sender = self.sender.clone();
            thread::spawn(move || {
                if let Ok(client) = Self::handle_client(stream)
                    && sender.send(client).is_err()
                {
                    *connection.lock().unwrap() = Connection::End;
                }
            });
        }
    }

    fn handle_client(stream: TcpStream) -> Result<chat::Client, ()> {
        match chat::Client::try_new(stream) {
            Ok(client) => {
                eprintln!("New client at the address {}", client.ip());
                Ok(client)
            }
            Err(chat::ClientError::IO(e)) => {
                eprintln!("ERROR: IO error from client_handle {e}");
                Err(())
            }
            Err(e) => {
                eprintln!("ERROR: from client_handle {e:#?}");
                Err(())
            }
        }
    }
}

fn main() {
    let (server, receiver) = Server::bind_server();
    let state_server = Arc::clone(&server.connection);

    thread::spawn(move || {
        server.run();
    });

    let mut counter = 0;
    for client in receiver {
        eprintln!("Receiver: new client {}", client.ip());
        counter += 1;

        if counter == 5 {
            *state_server.lock().unwrap() = Connection::Drop;
        }
    }
}
