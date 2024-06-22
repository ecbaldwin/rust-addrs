#[derive(Debug)]
pub enum Error {
    InvalidLength,
    InvalidMask,
    TooMany,
}

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
