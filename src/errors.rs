/// enumerates the possible errors from methods in this crate
#[derive(Debug)]
pub enum Error {
    /// returned when an invalid length was given (i.e. >32 for IPv4 and >128 for IPv6)
    InvalidLength,
    /// returned when an invalid netmask was given (e.g. 255.255.0.255)
    InvalidMask,
    /// returned when counting addresses or prefixes overflows
    TooMany,
}

/// returned from methods in this crate
pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::InvalidLength => {
                write!(f, "length is more than the number bits in the address")
            }
            Error::InvalidMask => write!(f, "invalid netmask"),
            Error::TooMany => write!(f, "too many to count"),
        }
    }
}
