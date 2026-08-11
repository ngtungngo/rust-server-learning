use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
pub fn get_health(addr: SocketAddr) -> std::io::Result<String> {
    let mut stream = TcpStream::connect(addr)?;
    stream.write_all(b"GET /api/health HTTP/1.1\r\n\r\n")?;
    stream.shutdown(Shutdown::Write)?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf)?;
    Ok(buf)
}
