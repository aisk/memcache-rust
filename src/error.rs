use std::error;
use std::fmt;
use std::io;
use std::num;
use std::str;
use std::string;
use url;

#[derive(Debug, PartialEq)]
pub enum ClientError {
    KeyTooLong,
    Error(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::KeyTooLong => write!(f, "The provided key was too long."),
            ClientError::Error(s) => write!(f, "{}", s),
        }
    }
}

impl From<ClientError> for MemcacheError {
    fn from(err: ClientError) -> Self {
        MemcacheError::ClientError(err)
    }
}

#[derive(Debug)]
pub enum ServerError {
    BadMagic(u8),
    BadResponse(String),
    Error(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServerError::BadMagic(e) => write!(f, "Expected 0x81 as magic in response header, but found: {:x}", e),
            ServerError::BadResponse(s) => write!(f, "Unexpected: {} in response", s),
            ServerError::Error(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CommandError {
    KeyExists,
    KeyNotFound,
    ValueTooLarge,
    InvalidArguments,
    AuthenticationRequired,
    Unknown(u16),
    InvalidCommand,
}

impl MemcacheError {
    pub(crate) fn try_from(s: String) -> Result<String, MemcacheError> {
        if s == "ERROR\r\n" {
            Err(CommandError::InvalidCommand)?
        } else if s.starts_with("CLIENT_ERROR") {
            Err(ClientError::from(s))?
        } else if s.starts_with("SERVER_ERROR") {
            Err(ServerError::from(s))?
        } else if s == "NOT_FOUND\r\n" {
            Err(CommandError::KeyNotFound)?
        } else if s == "EXISTS\r\n" {
            Err(CommandError::KeyExists)?
        } else {
            Ok(s)
        }
    }
}

impl From<String> for ClientError {
    fn from(s: String) -> Self {
        ClientError::Error(s)
    }
}

impl From<String> for ServerError {
    fn from(s: String) -> Self {
        ServerError::Error(s)
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandError::KeyExists => write!(f, "Key already exists in the server."),
            CommandError::KeyNotFound => write!(f, "Key was not found in the server."),
            CommandError::ValueTooLarge => write!(f, "Value was too large."),
            CommandError::InvalidArguments => write!(f, "Invalid arguments provided."),
            CommandError::AuthenticationRequired => write!(f, "Authentication required."),
            CommandError::Unknown(code) => write!(f, "Unknown error occurred with code: {}.", code),
            CommandError::InvalidCommand => write!(f, "Invalid command sent to the server."),
        }
    }
}

impl From<u16> for CommandError {
    fn from(status: u16) -> CommandError {
        match status {
            0x0 => panic!("shouldn't reach here"),
            0x1 => CommandError::KeyNotFound,
            0x2 => CommandError::KeyExists,
            0x3 => CommandError::ValueTooLarge,
            0x4 => CommandError::InvalidArguments,
            0x20 => CommandError::AuthenticationRequired,
            e => CommandError::Unknown(e),
        }
    }
}

impl From<CommandError> for MemcacheError {
    fn from(err: CommandError) -> Self {
        MemcacheError::CommandError(err)
    }
}

impl From<ServerError> for MemcacheError {
    fn from(err: ServerError) -> Self {
        MemcacheError::ServerError(err)
    }
}

#[derive(Debug)]
pub enum ParseError {
    Bool(str::ParseBoolError),
    Int(num::ParseIntError),
    Float(num::ParseFloatError),
    String(string::FromUtf8Error),
    Url(url::ParseError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::Bool(ref e) => e.fmt(f),
            ParseError::Int(ref e) => e.fmt(f),
            ParseError::Float(ref e) => e.fmt(f),
            ParseError::String(ref e) => e.fmt(f),
            ParseError::Url(ref e) => e.fmt(f),
        }
    }
}

impl From<ParseError> for MemcacheError {
    fn from(err: ParseError) -> Self {
        MemcacheError::ParseError(err)
    }
}

impl From<string::FromUtf8Error> for MemcacheError {
    fn from(err: string::FromUtf8Error) -> MemcacheError {
        ParseError::String(err).into()
    }
}
impl From<num::ParseIntError> for MemcacheError {
    fn from(err: num::ParseIntError) -> MemcacheError {
        ParseError::Int(err).into()
    }
}

impl From<num::ParseFloatError> for MemcacheError {
    fn from(err: num::ParseFloatError) -> MemcacheError {
        ParseError::Float(err).into()
    }
}

impl From<url::ParseError> for MemcacheError {
    fn from(err: url::ParseError) -> MemcacheError {
        ParseError::Url(err).into()
    }
}

impl From<str::ParseBoolError> for MemcacheError {
    fn from(err: str::ParseBoolError) -> MemcacheError {
        ParseError::Bool(err).into()
    }
}

/// Stands for errors raised from rust-memcache
// TODO: implement From<String> for the new version
#[derive(Debug)]
pub enum MemcacheError {
    /// Error raised when the provided memcache URL is invalid
    #[cfg(feature = "tls")]
    BadURL(String),
    /// `std::io` related errors.
    IOError(io::Error),
    /// Client Errors
    ClientError(ClientError),
    /// Server Errors
    ServerError(ServerError),
    /// Command specific Errors
    CommandError(CommandError),
    #[cfg(feature = "tls")]
    OpensslError(openssl::ssl::HandshakeError<std::net::TcpStream>),
    /// Parse errors
    ParseError(ParseError),
}

impl fmt::Display for MemcacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MemcacheError::BadURL(ref s) => s.fmt(f),
            MemcacheError::IOError(ref err) => err.fmt(f),
            #[cfg(feature = "tls")]
            MemcacheError::OpensslError(ref err) => err.fmt(f),
            MemcacheError::ParseError(ref err) => err.fmt(f),
            MemcacheError::ClientError(ref err) => err.fmt(f),
            MemcacheError::ServerError(ref err) => err.fmt(f),
            MemcacheError::CommandError(ref err) => err.fmt(f),
        }
    }
}

impl error::Error for MemcacheError {
    fn description(&self) -> &str {
        match *self {
            #[cfg(feature = "tls")]
            MemcacheError::BadURL(ref s) => s.as_str(),
            MemcacheError::IOError(ref err) => err.description(),
            #[cfg(feature = "tls")]
            MemcacheError::OpensslError(ref err) => err.description(),
            // TODO: implement these.
            MemcacheError::ClientError(_) => "Client error",
            MemcacheError::ServerError(_) => "Server error",
            MemcacheError::ParseError(_) => "Parse error",
            MemcacheError::CommandError(_) => "Command error",
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            #[cfg(feature = "tls")]
            MemcacheError::BadURL(_) => None,
            MemcacheError::IOError(ref err) => err.source(),
            #[cfg(feature = "tls")]
            MemcacheError::OpensslError(ref err) => err.source(),
            // TODO: implement these.
            MemcacheError::ParseError(_) => None,
            MemcacheError::ClientError(_) => None,
            MemcacheError::ServerError(_) => None,
            MemcacheError::CommandError(_) => None,
        }
    }
}

impl From<io::Error> for MemcacheError {
    fn from(err: io::Error) -> MemcacheError {
        MemcacheError::IOError(err)
    }
}

#[cfg(feature = "tls")]
impl From<openssl::error::ErrorStack> for MemcacheError {
    fn from(err: openssl::error::ErrorStack) -> MemcacheError {
        MemcacheError::OpensslError(openssl::ssl::HandshakeError::<std::net::TcpStream>::from(err))
    }
}

#[cfg(feature = "tls")]
impl From<openssl::ssl::HandshakeError<std::net::TcpStream>> for MemcacheError {
    fn from(err: openssl::ssl::HandshakeError<std::net::TcpStream>) -> MemcacheError {
        MemcacheError::OpensslError(err)
    }
}
