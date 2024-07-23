#![deny(missing_docs)]

//! Provides IP address related types and data structures for the Rust programming language with a
//! clean and complete API and many nice features. The basic types are opaque, immutable,
//! comparable, space efficient, and defined as simple structs that don't require extra memory
//! allocation.
//!
//! First, a little history: I write a lot of go code in my job and I do a lot of virtual
//! networking. I work with IP Addresses regularly. I was unhappy with the implementation of IP
//! addresses [in the go standard library][gonet], and a few other go libraries, for a number of
//! reasons, so [I wrote my own][goaddrs].
//!
//! For now, programming in Rust is more a part of my hobby-life than my work-life but I wanted to
//! replicate the functionality of my go library in Rust for two reasons: because I think it could
//! enrich Rust and I want a good learning experience.
//!
//! [goaddrs]: https://pkg.go.dev/gopkg.in/addrs.v1
//! [gonet]: https://pkg.go.dev/net#IP

/// enumerates the possible errors from methods in this crate
#[derive(Debug)]
pub enum Error {
    /// returned when an invalid length was given (i.e. >32 for IPv4 and >128 for IPv6)
    InvalidLength,
    /// returned when an invalid netmask was given (e.g. 255.255.0.255)
    InvalidMask,
    /// returned when counting addresses or prefixes overflows
    TooMany,
    /// returned when parsing a prefix from a string fails
    ParseError(Option<Box<dyn std::error::Error>>),
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
            Error::ParseError(_) => write!(f, "prefix parsing failed"),
            Error::TooMany => write!(f, "too many to count"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ParseError(s) => match s {
                Some(e) => Some(&**e),
                None => None,
            },
            _ => None,
        }
    }
}

/// work with IPv4 address, prefixes, etc.
pub mod ipv4;
