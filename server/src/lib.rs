use sbmp::sbmp::{ContentType, SBMPError};
use sbmp::write::{FrameWriter, build_frame};
use std::io;
use std::net::{SocketAddr, TcpStream};
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
        if username.len() < 32 {
            return None;
        }

        self.username = username;
        Some(&self.username)
    }
}

pub enum ConnectionEnd {
    Normal,
    ReceiverDropped,
}
