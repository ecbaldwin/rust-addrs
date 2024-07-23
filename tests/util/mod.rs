use std::{
    fmt::Display,
    net::{AddrParseError, Ipv4Addr},
    str::FromStr,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Address {
    octets: [u8; 4],
}

impl Address {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self {
            octets: [a, b, c, d],
        }
    }

    pub const LOCALHOST: Address = Address {
        octets: [127, 0, 0, 1],
    };
}

// I want to add this trait bound to the main Address trait
pub trait ToArray: Into<[u8; 4]> {}
impl ToArray for Address {}

impl addrs::ipv4::Address for Address {
    type UI = u32;

    fn octets(&self) -> [u8; 4] {
        self.octets
    }
}

impl From<Address> for u32 {
    fn from(value: Address) -> Self {
        u32::from_be_bytes(value.octets)
    }
}

impl From<u32> for Address {
    fn from(value: u32) -> Self {
        Self {
            octets: value.to_be_bytes(),
        }
    }
}

impl From<[u8; 4]> for Address {
    fn from(value: [u8; 4]) -> Self {
        Self { octets: value }
    }
}

impl From<Address> for [u8; 4] {
    fn from(ip: Address) -> Self {
        ip.octets
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ip = Ipv4Addr::from(self.octets);
        ip.fmt(f)
    }
}

impl FromStr for Address {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Ipv4Addr::from_str(s) {
            Ok(ip) => Ok(Self {
                octets: ip.octets(),
            }),
            Err(e) => Err(e),
        }
    }
}

impl std::ops::BitAnd for Address {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let lhs: u32 = self.into();
        let rhs: u32 = rhs.into();
        Self::Output {
            octets: (lhs & rhs).to_be_bytes(),
        }
    }
}
impl std::ops::BitOr for Address {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        let lhs: u32 = self.into();
        let rhs: u32 = rhs.into();
        Self::Output {
            octets: (lhs | rhs).to_be_bytes(),
        }
    }
}
impl std::ops::Not for Address {
    type Output = Self;

    fn not(self) -> Self::Output {
        let lhs: u32 = self.into();
        Self::Output {
            octets: (!lhs).to_be_bytes(),
        }
    }
}

impl core::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        core::fmt::Debug::fmt(&Ipv4Addr::from(self.octets), f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Prefix {
    address: Address,
    length: u8,
}

impl addrs::ipv4::Prefix for Prefix {
    type Address = Address;

    fn address(&self) -> Self::Address {
        self.address
    }
    fn length(&self) -> u8 {
        self.length
    }
    unsafe fn unsafe_new(address: Self::Address, length: u8) -> Self {
        Self { address, length }
    }
}

impl std::fmt::Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.address)?;
        write!(f, "/")?;
        write!(f, "{}", self.length)
    }
}

impl std::str::FromStr for Prefix {
    type Err = ipnet::AddrParseError;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        let net = ipnet::Ipv4Net::from_str(s)?;
        let addr = net.addr();
        let array: [u8; 4] = addr.octets();
        let addr: Address = array.into();
        Ok(unsafe { addrs::ipv4::Prefix::unsafe_new(addr, net.prefix_len()) })
    }
}

#[allow(dead_code)]
pub fn a(s: &str) -> Address {
    s.parse().expect("bad ip")
}

#[allow(dead_code)]
pub fn p(s: &str) -> Prefix {
    s.parse().expect("bad prefix")
}

#[allow(dead_code)]
pub fn assert_result<T: Eq + std::fmt::Debug, E: ToString>(
    expected: Result<T, E>,
    actual: Result<T, E>,
) {
    match expected {
        Ok(expected) => {
            assert!(actual
                .inspect(|actual| {
                    assert_eq!(expected, *actual);
                })
                .is_ok());
        }
        Err(expected) => {
            assert!(actual
                .inspect_err(|actual| {
                    assert_eq!(expected.to_string(), actual.to_string());
                })
                .is_err());
        }
    }
}
