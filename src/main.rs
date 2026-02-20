use std::{collections::HashMap, sync::Arc, thread};

use chat::{Client, Server};

#[derive(Default)]
struct Room {
    clients: HashMap<usize, Client>,
    capacity: usize,
}

impl Room {
    fn new(capacity: usize) -> Self {
        Self {
            clients: HashMap::new(),
            capacity,
        }
    }

    fn is_full(&self) -> bool {
        self.clients.len() == self.capacity
    }
}

fn main() {
    let (server, receiver) = Server::bind_server();
    let state_server = Arc::clone(&server.connection);

    thread::spawn(move || {
        server.run();
    });

    let mut room = Room::new(42);

    for mut client in receiver {
        if room.is_full() {}
    }
}
