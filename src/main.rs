use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
};

fn main() {
    let (listener, server_address) = start_server();

    println!("Serving on {server_address}");

    run_server(listener);
}

fn start_server() -> (TcpListener, SocketAddr) {
    let addr = "0.0.0.0:1337";
    eprintln!("Binding server at {addr}");
    let listener = TcpListener::bind(addr).expect("ERROR: Failed to bind server");
    let server_address = listener
        .local_addr()
        .expect("ERROR: Failed to find server address");

    (listener, server_address)
}

fn run_server(listener: TcpListener) {
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

fn handle_client(stream: TcpStream) {
    let client = match chat::Client::try_new(stream) {
        Ok(client) => client,
        Err(chat::ClientError::IO(e)) => {
            eprintln!("ERROR: from client_handle {e}");
            return;
        }
        Err(e) => {
            eprintln!("ERROR: from client_handle {e:#?}");
            return;
        }
    };

    eprintln!(
        "New client {} at the address {}",
        client.username(),
        client.ip()
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

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use super::*;

    const ADDRESS: &str = "0.0.0.0:1337";

    #[test]
    fn handle_256_clients() {
        const NUM_CONNECTIONS: usize = 256;
        let mut v = vec![];

        for _i in 0..NUM_CONNECTIONS {
            let t = thread::spawn(move || {
                let stream = TcpStream::connect(ADDRESS).expect("ERROR: Try start the server");

                thread::sleep(Duration::from_secs(5));

                stream.shutdown(std::net::Shutdown::Both).unwrap();
            });
            v.push(t);
        }

        for t in v {
            t.join().unwrap();
        }
    }
}
