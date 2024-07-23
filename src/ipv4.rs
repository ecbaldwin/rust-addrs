use std::ops::RangeInclusive;

use crate::errors::{Error, Result};

/// Defines minimum requirements of an ipv4 address for this crate
///
/// The purpose of this trait is not to replace nor even add to [`std::net::Ipv4Addr`]. It is well
/// thought out and (nearly) complete enough for my purposes[^1]. For this reason, this crate
/// doesn't even provide a type alias for it.
///
/// This trait is mostly for use within this crate to formalize its good properties on which this
/// crate may depend. We also provide a minimal set of integration tests which are meant as a
/// sanity check on the its behavior, not a comprehensive test suite.
///
/// This trait also servers to limit the touch points this crate has on that type.
///
/// [^1]: One minor exception is that I wish it implemented Into<[u8; 4]>. There is
/// [`std::net::Ipv4Addr::octets`] but it is more awkward.
pub trait Address:
    Eq
    + Ord
    + Clone
    + Copy
    + From<u32>
    + Into<u32>
    + From<[u8; 4]>
//     + Into<[u8; 4]>
    + std::string::ToString
    + std::str::FromStr
    + std::ops::BitAnd<Output = Self>
    + std::ops::BitOr<Output = Self>
    + std::ops::Not<Output = Self>
    + Sized
    + Send
    + Sync
    + Unpin
{
    /// formalize that all v4 address are 32 bits
    const BITS: u8 = 32;

    /// returns the bytes of the address in network order
    ///
    /// in lieu of implementing Into<[u8; 4]>, this allows this crate to easily get at the
    /// underlying byte array.
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::Address;
    /// # use std::net::Ipv4Addr;
    /// fn check<A: Address>(address: A) {
    ///     assert_eq!([1, 2, 3, 4], address.octets());
    /// }
    /// check("1.2.3.4".parse::<Ipv4Addr>().unwrap());
    /// ```
    fn octets(&self) -> [u8; 4];
}

/// Defines minimum requirements of an ipv4 prefix for this crate and provides implementations of
/// new methods.
///
/// A prefix -- otherwise commonly referred to as a subnet, network, or CIDR -- combines an address
/// with a prefix length to create a set of IP addresses which can be trivially matched in software
/// or hardware with simple bitwise integer operations. Once matched, the addresses belonging to
/// the set can be handled in a common way. This is basis for forwarding packets on the internet.
///
/// The Rust standard library does not provide an implementation. However, [`ipnet::Ipv4Net`]
/// appears to have some popularity in other network related crates. Therefore, an implementation
/// of this trait is provided for it and it is used in a basic set of integration tests. Many of
/// the required and provided methods are similar to ones it provides but the semantics were
/// different enough from what I wanted that I reimplemented them with trivial wrappers.
pub trait Prefix: Eq + std::str::FromStr + std::string::ToString {
    /// the type of IP address associated with this prefix
    type Address: Address;

    /// returns the address part of the Prefix, including host bits
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::{Prefix};
    /// let prefix = "1.2.3.4/24".parse::<ipnet::Ipv4Net>().unwrap();
    /// assert_eq!("1.2.3.4", prefix.address().to_string());
    /// ```
    fn address(&self) -> Self::Address;

    /// returns the prefix length which is the number of leading 1s in the netmask
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::Prefix;
    /// let prefix = "1.2.3.4/23".parse::<ipnet::Ipv4Net>().unwrap();
    /// assert_eq!(23, prefix.length());
    /// ```
    fn length(&self) -> u8;

    /// returns a new Prefix without checking length for when I know what I'm doing
    unsafe fn unsafe_new(ip: Self::Address, length: u8) -> Self;

    /// returns the prefix for the given address combined with the given prefix length. If the
    /// length is greater than 32 then [`Error::InvalidLength`] is returned.
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::Prefix;
    /// # use std::net::Ipv4Addr;
    /// let ip = Ipv4Addr::new(1,2,3,4);
    /// let prefix: ipnet::Ipv4Net = Prefix::from_address_length(ip, 25).unwrap();
    /// assert_eq!("1.2.3.4/25", prefix.to_string());
    /// ```
    fn from_address_length(ip: Self::Address, length: u8) -> Result<Self> {
        match length {
            length if length < Self::Address::BITS => Ok(unsafe { Self::unsafe_new(ip, length) }),
            _ => Err(Error::InvalidLength),
        }
    }

    /// returns the prefix for the given address combined with the given mask. The mask must be an
    /// instance of [`Address`] where anywhere from 0 to 32 left-most bits are all 1s followed by
    /// all 0s on the right. If the mask is invalid, [`Error::InvalidMask`] is returned.
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::Prefix;
    /// # use std::net::Ipv4Addr;
    /// let ip = Ipv4Addr::new(8,7,6,5);
    /// let mask = Ipv4Addr::new(255,255,252,0);
    /// let prefix: ipnet::Ipv4Net = Prefix::from_address_mask(ip, mask).unwrap();
    /// assert_eq!("8.7.6.5/22", prefix.to_string());
    fn from_address_mask(ip: Self::Address, mask: Self::Address) -> Result<Self> {
        let mask: u32 = mask.into();
        let length = mask.leading_ones() as u8;
        match length + mask.trailing_zeros() as u8 == Self::Address::BITS {
            true => Ok(unsafe { Self::unsafe_new(ip, length) }),
            false => Err(Error::InvalidMask),
        }
    }

    /// returns a new Address with `1s` in the first `length` bits and then `0s` representing the
    /// network mask for this prefix
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::Prefix;
    /// let prefix = "1.2.3.4/26".parse::<ipnet::Ipv4Net>().unwrap();
    /// assert_eq!("255.255.255.192", prefix.mask().to_string());
    /// ```
    fn mask(&self) -> Self::Address {
        match self.length() {
            0 => 0,
            s => 0xffffffff << Self::Address::BITS - s,
        }
        .into()
    }

    /// returns a new Prefix with the host bits zeroed out so that only the bits in the `network`
    /// part of the prefix can be non-zero. Note that this method ignores special cases where a
    /// network address might not make sense like in a host route or point-to-point prefix (/32 and
    /// /31). It just does the math.
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::Prefix;
    /// let prefix = "1.2.3.234/26".parse::<ipnet::Ipv4Net>().unwrap();
    /// assert_eq!("1.2.3.192", Prefix::network(&prefix).to_string());
    /// ```
    fn network(&self) -> Self::Address {
        self.address() & self.mask()
    }

    /// returns a new Prefix with the network bits zeroed out so that only the bits in the
    /// `host` part of the prefix can be non-zero.
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::Prefix;
    /// fn check<P: Prefix>(prefix: P) {
    /// }
    ///
    /// let prefix = "1.2.3.234/26".parse::<ipnet::Ipv4Net>().unwrap();
    /// assert_eq!(42u32, prefix.host().into());
    /// ```
    fn host(&self) -> Self::Address {
        self.address() & !self.mask()
    }

    /// returns a new Prefix with all the host bits set to 1s. Note that this method ignores
    /// special cases where a broadcast address might not make sense like in a host route or
    /// point-to-point prefix (/32 and /31). It just does the math.
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::Prefix;
    /// let prefix = "1.2.3.1/24".parse::<ipnet::Ipv4Net>().unwrap();
    /// assert_eq!("1.2.3.255", Prefix::broadcast(&prefix).to_string());
    /// ```
    fn broadcast(&self) -> Self::Address {
        self.address() | !self.mask()
    }

    /// returns two prefixes that partition this prefix into two equal halves. If the prefix is a
    /// host route (/32), then None is returned.
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::Prefix;
    /// let prefix: ipnet::Ipv4Net = "1.2.3.0/24".parse().unwrap();
    /// let (a, b) = prefix.halves().unwrap();
    /// assert_eq!("1.2.3.0/25", a.to_string());
    /// assert_eq!("1.2.3.128/25", b.to_string());
    /// ```
    fn halves(&self) -> Option<(Self, Self)> {
        match self.length() {
            length if length < Self::Address::BITS => {
                let left: u32 = self.network().address().into();
                let right = left | (0x80000000 >> length);
                Some((
                    unsafe { Self::unsafe_new(left.into(), length + 1) },
                    unsafe { Self::unsafe_new(right.into(), length + 1) },
                ))
            }
            _ => None,
        }
    }

    /// returns an inclusive range of IP addresses equivalent to the range of addresses contained
    /// within this Prefix. The range is not open-ended so that the entire IP range can be
    /// represented.
    ///
    /// Example
    /// ```
    /// # use addrs::ipv4::{Prefix, Set};
    /// # use std::net::Ipv4Addr;
    ///
    /// // Contains self
    /// let net: ipnet::Ipv4Net = "192.168.0.0/24".parse().unwrap();
    /// let range = net.as_range_i();
    /// let ip_yes: Ipv4Addr = "192.168.0.1".parse().unwrap();
    /// let ip_no: Ipv4Addr = "192.168.1.0".parse().unwrap();
    /// assert_eq!(net.contains(&ip_yes), range.contains(&ip_yes));
    /// assert_eq!(net.contains(&ip_no), range.contains(&ip_no));
    /// ```
    fn as_range_i(&self) -> RangeInclusive<Self::Address> {
        RangeInclusive::new(self.network().address(), self.broadcast().address())
    }
}

impl<T, P> Set for P
where
    T: Address,
    P: Prefix<Address = T>,
{
    type Address = T;

    fn num_prefixes(&self, length: u8) -> Result<u32> {
        match length {
            length if length < self.length() => Ok(0),
            length if Self::Address::BITS < length => Err(Error::InvalidLength),
            length => {
                let p = (length - self.length()).into();
                match 2u32.checked_pow(p) {
                    Some(c) => Ok(c),
                    None => Err(Error::TooMany),
                }
            }
        }
    }

    fn contains<P2: Prefix>(&self, other: &P2) -> bool {
        use prefix_private::{Cmp, PrefixOrd::*};

        match self.cmp(other) {
            (Same | Contains, _, _, _) => true,
            _ => false,
        }
    }
}

impl<T> Set for RangeInclusive<T>
where
    T: Address,
{
    type Address = T;

    fn num_prefixes(&self, length: u8) -> Result<u32> {
        match length {
            length if T::BITS < length => Err(Error::InvalidLength),
            _ => {
                let start: u32 = (*self.start()).into();
                let end: u32 = (*self.end()).into();
                if end < start {
                    return Ok(0);
                }

                let xor = start ^ end;
                let zeros = xor.leading_zeros();
                let (mask, pivot) = match u32::MAX.checked_shl(u32::BITS - zeros) {
                    Some(u32::MAX) => (u32::MAX, 0), // zeroes is 32
                    Some(m) => (m, 1 << (u32::BITS - (zeros + 1))),
                    None => (0, 1 << (u32::BITS - 1)), // zeroes is 0
                };
                let middle = (start & mask) | pivot;
                let size = 2u32.pow(u32::BITS - length as u32);
                Ok((middle - start) / size + (end - middle + 1) / size)
            }
        }
    }

    fn contains<P2: Prefix<Address = T>>(&self, other: &P2) -> bool {
        // This implementation will need to change when Prefix changes to Set for the containee
        return RangeInclusive::<T>::contains::<T>(self, &other.network().address())
            && RangeInclusive::<T>::contains::<T>(self, &other.broadcast().address());
    }
}

impl<T> Prefix for T
where
    T: Address,
{
    type Address = T;

    fn address(&self) -> Self::Address {
        *self
    }
    fn length(&self) -> u8 {
        Self::BITS
    }

    unsafe fn unsafe_new(ip: Self::Address, _length: u8) -> Self {
        ip
    }
}

/// Defines minimum requirements of an ipv4 set for this crate.
pub trait Set {
    /// the type of IP address associated with this set
    type Address: Address;

    /// returns the number of addresses in the set.
    /// It ignores any bits set in the host part of the address. In the case of 0 prefix length, it
    /// returns [`Error::TooMany`].
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::Set;
    /// let prefix = "1.2.3.0/25".parse::<ipnet::Ipv4Net>().unwrap();
    /// assert_eq!(128, prefix.num_addresses().unwrap());
    /// ```
    fn num_addresses(&self) -> Result<u32> {
        self.num_prefixes(Self::Address::BITS)
    }

    /// returns the number of prefixes of the given length contained in this set. If the number
    /// would overflow a [`u32`] it returns [`Error::TooMany`]. If >32 is passed for length then
    /// [`Error::InvalidLength`] is returned.
    ///
    /// # Examples
    /// ```
    /// # use addrs::ipv4::Set;
    /// let prefix = "1.2.3.0/25".parse::<ipnet::Ipv4Net>().unwrap();
    /// assert_eq!(4, prefix.num_prefixes(27).unwrap());
    /// ```
    /// When the prefix length specifies a prefix having more than one address (e.g. a /24 in IPv4
    /// contains 256 addresses) then only properly aligned, wholly contained, prefixes of that size
    /// are counted. See the following example noting that there are many addresses on both ends of
    /// the range that cannot be included in a prefix of length 24 because of alignment.
    /// ```
    /// # use addrs::ipv4::Set;
    /// # use std::net::Ipv4Addr;
    /// let from = "10.223.255.1".parse::<Ipv4Addr>().unwrap();
    /// let to = "10.225.0.254".parse::<Ipv4Addr>().unwrap();
    /// let range = from..=to;
    /// assert_eq!(256, range.num_prefixes(24).unwrap());
    /// ```
    fn num_prefixes(&self, length: u8) -> Result<u32>;

    /// returns true if the set is empty
    /// # Example
    /// ```
    /// # use addrs::ipv4::Set;
    /// let prefix = "1.2.3.0/25".parse::<ipnet::Ipv4Net>().unwrap();
    /// assert!(!prefix.is_empty());
    /// ```
    fn is_empty(&self) -> bool {
        match self.num_addresses() {
            Ok(0) => true,
            _ => false,
        }
    }

    /// returns true if the given containee is wholly contained within this Prefix. If the two
    /// Prefixes are equal, true is returned. The host bits in the address are ignored when testing
    /// containership.
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::{Prefix, Set};
    /// # use std::net::Ipv4Addr;
    /// // Contains self
    /// let net: ipnet::Ipv4Net = "192.168.0.0/24".parse().unwrap();
    /// assert!(net.contains(&net));
    ///
    /// // Nets
    /// let net_yes: ipnet::Ipv4Net = "192.168.0.0/25".parse().unwrap();
    /// let net_no: ipnet::Ipv4Net = "192.168.0.0/23".parse().unwrap();
    /// assert!(net.contains(&net_yes));
    /// assert!(!net.contains(&net_no));
    ///
    /// // IPs
    /// let ip_yes: Ipv4Addr = "192.168.0.1".parse().unwrap();
    /// let ip_no: Ipv4Addr = "192.168.1.0".parse().unwrap();
    /// assert!(net.contains(&ip_yes));
    /// assert!(!net.contains(&ip_no));
    /// ```
    fn contains<P2: Prefix<Address = Self::Address>>(&self, other: &P2) -> bool;
}

// https://stackoverflow.com/questions/53204327/how-to-have-a-private-part-of-a-trait
mod prefix_private;

/// implements traits for external types
pub mod implementations;
