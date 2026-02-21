use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};

pub enum Message {
    Broadcast(String),
    NewClient(Client),
    Drop(SocketAddr),
}

pub struct Client {
    ip: SocketAddr,
    writer: TcpStream,
}

impl Drop for Client {
    fn drop(&mut self) {
        eprintln!("INFO: dropping client with address {}", self.ip);
        let _ = self.writer.shutdown(std::net::Shutdown::Both);
    }
}

impl Client {
    pub fn try_new(stream: TcpStream) -> Result<Client, io::Error> {
        let ip = stream.peer_addr()?;

        Ok(Client::new(stream, ip))
    }

    fn new(writer: TcpStream, ip: SocketAddr) -> Self {
        Self { ip, writer }
    }

    pub fn write(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(buf)
    }

    pub fn ip(&self) -> SocketAddr {
        self.ip
    }
}

pub enum ConnectionEnd {
    Normal,
    ReceiverDropped,
}

pub struct Reader {
    reader: BufReader<TcpStream>,
    max_len: u8,
}

impl Reader {
    pub fn new(stream: TcpStream, max_len: u8) -> Self {
        Self {
            reader: BufReader::new(stream),
            max_len,
        }
    }

    pub fn read_line(&mut self) -> Result<String, io::Error> {
        let mut msg = String::with_capacity(self.max_len as usize);

        match self
            .reader
            .by_ref()
            .take(self.max_len as u64)
            .read_line(&mut msg)
        {
            Ok(0) => Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
            Err(e) => Err(e),
            Ok(_) => Ok(msg),
        }
    }
}
