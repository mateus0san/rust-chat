use sbmp::sbmp::{ContentType, SBMPError};
use sbmp::write::{FrameWriter, build_frame};
use std::io;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

pub enum Message {
    Broadcast(String),
    NewClient(Client),
    Drop(SocketAddr),
}

pub struct Client {
    username: String,
    ip: SocketAddr,
    writer: FrameWriter<TcpStream>,
}

impl Drop for Client {
    fn drop(&mut self) {
        eprintln!("INFO: dropping client with address {}", self.ip);
        let stream = self.writer.get_ref();
        let _ = stream.shutdown(std::net::Shutdown::Both);
    }
}

impl Client {
    pub fn try_new(stream: TcpStream) -> io::Result<Client> {
        let ip = stream.peer_addr()?;

        stream.set_read_timeout(Some(Duration::from_mins(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(15)))?;

        Ok(Client::new(stream, ip))
    }

    fn new(stream: TcpStream, ip: SocketAddr) -> Self {
        Self {
            username: String::from("Guest"),
            ip,
            writer: FrameWriter::new(stream),
        }
    }

    pub fn write(&mut self, s: &str) -> Result<(), SBMPError> {
        let frame = build_frame(ContentType::UTF8, s.as_bytes())?;
        self.writer.write_frame(frame)
    }

    pub fn ip(&self) -> SocketAddr {
        self.ip
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn set_username(&mut self, username: String) -> Option<&str> {
        let username = username.trim();
        if username.len() >= 32 || username.is_empty() {
            return None;
        }

        self.username = username.to_string();
        Some(&self.username)
    }
}

pub enum ConnectionEnd {
    Normal,
    ReceiverDropped,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn build(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let job = receiver.lock().unwrap().recv().unwrap();
                job();
            }
        });

        Worker { id, thread }
    }
}
