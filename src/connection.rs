use error::MemcacheError;
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::time::Duration;
use udp_stream::UdpStream;
#[cfg(unix)]
use url::Host;
use url::Url;

enum Stream {
    TcpStream(TcpStream),
    UdpSocket(UdpStream),
    #[cfg(unix)]
    UnixStream(UnixStream),
}

/// a connection to the memcached server
pub struct Connection {
    stream: Stream,
    pub url: String,
}

impl Connection {
    pub fn connect(addr: &str) -> Result<Self, MemcacheError> {
        let addr = match Url::parse(addr) {
            Ok(v) => v,
            Err(_) => return Err(MemcacheError::ClientError("Invalid memcache URL".into())),
        };
        if addr.scheme() != "memcache" {
            return Err(MemcacheError::ClientError(
                "memcache URL should start with 'memcache://'".into(),
            ));
        }

        let is_udp = addr
            .query_pairs()
            .any(|(ref k, ref v)| k == "udp" && v == "true");

        if is_udp {
            let udp_stream = Stream::UdpSocket(UdpStream::new(addr.clone())?);
            return Ok(Connection {
                url: addr.into_string(),
                stream: udp_stream,
            });
        }

        #[cfg(unix)]
        {
            if addr.host() == Some(Host::Domain("")) && addr.port() == None {
                let stream = UnixStream::connect(addr.path())?;
                return Ok(Connection {
                    url: addr.into_string(),
                    stream: Stream::UnixStream(stream),
                });
            }
        }

        let stream = TcpStream::connect(addr.clone())?;

        let disable_tcp_nodelay = addr
            .query_pairs()
            .any(|(ref k, ref v)| k == "tcp_nodelay" && v == "false");
        if !disable_tcp_nodelay {
            stream.set_nodelay(true)?;
        }
        let timeout = addr.query_pairs()
            .find(|&(ref k, ref _v)| k == "timeout")
            .and_then(|(ref _k, ref v)| v.parse::<u64>().ok())
            .map(Duration::from_secs);
        if timeout.is_some() {
            stream.set_read_timeout(timeout)?;
            stream.set_write_timeout(timeout)?;
        }
        return Ok(Connection {
            url: addr.into_string(),
            stream: Stream::TcpStream(stream),
        });
    }

    pub(crate) fn set_read_timeout(&mut self, timeout: Option<Duration>) -> Result<(), MemcacheError> {
        if  let Stream::TcpStream(ref mut conn) =  self.stream {
            conn.set_read_timeout(timeout)?;
        }
        Ok(())
    }

    pub(crate) fn set_write_timeout(&mut self, timeout: Option<Duration>) -> Result<(), MemcacheError> {
        if  let Stream::TcpStream(ref mut conn) =  self.stream {
            conn.set_write_timeout(timeout)?;
        }
        Ok(())
    }
}

impl Read for Connection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.stream {
            Stream::TcpStream(ref mut stream) => stream.read(buf),
            Stream::UdpSocket(ref mut stream) => stream.read(buf),
            #[cfg(unix)]
            Stream::UnixStream(ref mut stream) => stream.read(buf),
        }
    }
}

impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.stream {
            Stream::TcpStream(ref mut stream) => stream.write(buf),
            Stream::UdpSocket(ref mut stream) => stream.write(buf),
            #[cfg(unix)]
            Stream::UnixStream(ref mut stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.stream {
            Stream::TcpStream(ref mut stream) => stream.flush(),
            Stream::UdpSocket(ref mut stream) => stream.flush(),
            #[cfg(unix)]
            Stream::UnixStream(ref mut stream) => stream.flush(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tcp_nodelay() {
        super::Connection::connect("memcache://localhost:12345?tcp_nodelay=true").unwrap();
    }
}
