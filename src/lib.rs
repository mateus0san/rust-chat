use std::{
    io::{self, BufReader, Read, Write},
    net::TcpStream,
};

pub struct Client {
    username: String,
    writer: TcpStream,
    reader: BufReader<TcpStream>,
    ip: String,
}

#[derive(Debug)]
pub enum ClientError {
    IO(io::Error),
    Validation(std::string::FromUtf8Error),
}

impl From<io::Error> for ClientError {
    fn from(error: io::Error) -> Self {
        ClientError::IO(error)
    }
}

impl From<std::string::FromUtf8Error> for ClientError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        ClientError::Validation(error)
    }
}

impl Client {
    pub fn try_new(stream: TcpStream) -> Result<Self, ClientError> {
        let ip = stream.peer_addr()?.to_string();

        let writer = stream.try_clone()?;
        let reader = BufReader::new(stream);

        Ok(Client::new(writer, reader, ip))
    }

    pub fn set_username(&mut self) -> Result<(), ClientError> {
        self.writer
            .write_all("Type your username (max 16 characters): ".as_bytes())?;

        const MAX_USERNAME_LEN: usize = 16;
        let mut username = Vec::with_capacity(MAX_USERNAME_LEN);

        self.reader.read_exact(&mut username)?;

        Ok(())
    }

    fn new(writer: TcpStream, reader: BufReader<TcpStream>, ip: String) -> Self {
        Self {
            username: String::from("Guest"),
            writer,
            reader,
            ip,
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn ip(&self) -> &str {
        &self.ip
    }
}
