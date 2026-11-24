use std::{fmt, io};

#[derive(Debug)]
pub enum RpcError {
    DiscordNotFound,
    Closed(String),
    Rejected { code: i64, message: String },
    Protocol(&'static str),
    Io(io::Error),
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DiscordNotFound => write!(f, "Discord is not running"),
            Self::Closed(reason) => write!(f, "Discord closed the connection: {reason}"),
            Self::Rejected { code, message } => write!(f, "Discord refused ({code}): {message}"),
            Self::Protocol(reason) => write!(f, "unexpected data from Discord: {reason}"),
            Self::Io(error) => write!(f, "connection failed: {error}"),
        }
    }
}

impl std::error::Error for RpcError {}

impl From<io::Error> for RpcError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
