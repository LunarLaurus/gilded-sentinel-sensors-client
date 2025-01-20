use std::io::{self, Write};
use std::net::TcpStream;

pub fn send_data_to_server(data: &str, server: &str) -> io::Result<()> {
    let mut stream = TcpStream::connect(server)?;
    stream.write_all(data.as_bytes())?;
    Ok(())
}
