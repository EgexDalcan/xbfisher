use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("parsing error")]
    ParsingError,
    #[error("invalid tcp return message")]
    InvalidTCPReturn,
    #[error("invalid tcp communication")]
    InvalidTCPCommunication,
    #[error("invalid size")]
    InvalidSize,
    #[error("too small header")]
    TooSmallHeader,
    #[error("invalid header size")]
    InvalidHeaderSize,
    #[error("invalid version")]
    InvalidVersion,
    #[error("unknown protocol")]
    UnknownProtocol,
    #[error("invalid packet")]
    InvalidPacket,
    #[error("invalid procotol")]
    InvalidProtocol,
    #[error("internal error")]
    InternalError,
    #[error("Decode V4 error occurred while processing the IPv4 packet.")]
    DecodeV4Error,
    #[error("Decode echo reply error occurred while processing the ICMP echo reply.")]
    DecodeEchoReplyError,
    #[error("io error: {error}")]
    IoError {
        #[from]
        #[source]
        error: ::std::io::Error,
    },
}