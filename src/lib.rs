use std::io::{self, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

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
    pub fn try_new(stream: TcpStream) -> io::Result<Client> {
        let ip = stream.peer_addr()?;

        stream.set_read_timeout(Some(Duration::from_secs(15)))?;
        stream.set_write_timeout(Some(Duration::from_secs(15)))?;

        Ok(Client::new(stream, ip))
    }

    fn new(writer: TcpStream, ip: SocketAddr) -> Self {
        Self { ip, writer }
    }

    pub fn write(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_all(s.as_bytes())
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
}

impl Reader {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            reader: BufReader::new(stream),
        }
    }

    pub fn read_frame(&mut self) -> io::Result<Vec<u8>> {
        let len = self.read_header()?;

        if len > 8 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Frame too large",
            ));
        }

        let mut frame = vec![0u8; len as usize];

        self.reader.read_exact(&mut frame)?;

        Ok(frame)
    }

    fn read_header(&mut self) -> io::Result<u32> {
        // reading 4 bytes, content size
        let mut header = [0u8; 4];

        self.reader.read_exact(&mut header)?;

        Ok(u32::from_be_bytes(header))
    }
}
