use std::{net::Ipv4Addr, ops::RangeInclusive};

use crate::errors::{Error, Result};

/// Defines minimum requirements of an ipv4 address for this crate
///
/// The purpose of this trait is not to replace nor even add to [`Ipv4Addr`]. It is well thought
/// out and (nearly) complete enough for my purposes[^1]. For this reason, this crate doesn't even
/// provide a type alias for it.
///
/// This trait is mostly for use within this crate to formalize its good properties on which this
/// crate may depend. We also provide a minimal set of integration tests which are meant as a
/// sanity check on the its behavior, not a comprehensive test suite.
///
/// This trait also servers to limit the touch points this crate has on that type.
///
/// [^1]: One minor exception is that I wish it implemented Into<[u8; 4]>. There is
/// [`Ipv4Addr::octets`] but it is more awkward.
pub trait Address:
    Eq
    + Ord
    + From<u32>
    + Into<u32>
    + From<[u8; 4]>
//     + Into<[u8; 4]>
    + std::string::ToString
    + std::str::FromStr
    + std::ops::BitAnd<Output = Self>
    + std::ops::BitOr<Output = Self>
    + std::ops::Not<Output = Self>
    + Copy
    + Clone
    + Send
    + Sized
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

impl Address for std::net::Ipv4Addr {
    fn octets(&self) -> [u8; 4] {
        self.octets()
    }
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
    /// # use addrs::ipv4::{Ipv4Prefix, Prefix};
    /// let prefix = "1.2.3.4/24".parse::<Ipv4Prefix>().unwrap();
    /// assert_eq!("1.2.3.4", prefix.address().to_string());
    /// ```
    fn address(&self) -> Self::Address;
    /// returns the prefix length which is the number of leading 1s in the netmask
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::{Ipv4Prefix, Prefix};
    /// let prefix = "1.2.3.4/23".parse::<Ipv4Prefix>().unwrap();
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
    /// # use addrs::ipv4::{Ipv4Prefix, Prefix};
    /// # use std::net::Ipv4Addr;
    /// let ip = Ipv4Addr::new(1,2,3,4);
    /// let prefix: Ipv4Prefix = Prefix::from_address_length(ip, 25).unwrap();
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
    /// # use addrs::ipv4::{Ipv4Prefix, Prefix};
    /// # use std::net::Ipv4Addr;
    /// let ip = Ipv4Addr::new(8,7,6,5);
    /// let mask = Ipv4Addr::new(255,255,252,0);
    /// let prefix: Ipv4Prefix = Prefix::from_address_mask(ip, mask).unwrap();
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
    /// # use addrs::ipv4::{Ipv4Prefix, Prefix};
    /// let prefix = "1.2.3.4/26".parse::<Ipv4Prefix>().unwrap();
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
    /// # use addrs::ipv4::{Ipv4Prefix, Prefix};
    /// let prefix = "1.2.3.234/26".parse::<Ipv4Prefix>().unwrap();
    /// assert_eq!("1.2.3.192/26", prefix.network().to_string());
    /// ```
    fn network(&self) -> Self {
        let address = self.address() & self.mask();
        unsafe { Self::unsafe_new(address, self.length()) }
    }

    /// returns a new Prefix with the network bits zeroed out so that only the bits in the
    /// `host` part of the prefix can be non-zero.
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::{Ipv4Prefix, Prefix};
    /// fn check<P: Prefix>(prefix: P) {
    /// }
    ///
    /// let prefix = "1.2.3.234/26".parse::<Ipv4Prefix>().unwrap();
    /// assert_eq!("0.0.0.42/26", prefix.host().to_string());
    /// ```
    fn host(&self) -> Self {
        let address = self.address() & !self.mask();
        unsafe { Self::unsafe_new(address, self.length()) }
    }

    /// returns a new Prefix with all the host bits set to 1s. Note that this method ignores
    /// special cases where a broadcast address might not make sense like in a host route or
    /// point-to-point prefix (/32 and /31). It just does the math.
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::{Ipv4Prefix, Prefix};
    /// let prefix = "1.2.3.1/24".parse::<Ipv4Prefix>().unwrap();
    /// assert_eq!("1.2.3.255/24", prefix.broadcast().to_string());
    /// ```
    fn broadcast(&self) -> Self {
        let address = self.address() | !self.mask();
        unsafe { Self::unsafe_new(address, self.length()) }
    }

    /// returns two prefixes that partition this prefix into two equal halves. If the prefix is a
    /// host route (/32), then None is returned.
    ///
    /// # Example
    /// ```
    /// # use addrs::ipv4::{Ipv4Prefix, Prefix};
    /// let prefix: Ipv4Prefix = "1.2.3.0/24".parse().unwrap();
    /// let (a, b) = prefix.halves().unwrap();
    /// assert_eq!("1.2.3.0/25", a.to_string());
    /// assert_eq!("1.2.3.128/25", b.to_string());
    /// ```
    fn halves(&self) -> Option<(Self, Self)> {
        match self.length() {
            length if length < Self::Address::BITS => {
                let left = self.network().address().into();
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
    /// # use addrs::ipv4::{Ipv4Prefix, Prefix, Set};
    /// # use std::net::Ipv4Addr;
    ///
    /// // Contains self
    /// let net: Ipv4Prefix = "192.168.0.0/24".parse().unwrap();
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
        use prefix_private::Cmp;
        let (ord, _, _, _) = self.cmp(other);
        match ord {
            prefix_private::PrefixOrd::Same | prefix_private::PrefixOrd::Contains => true,
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

/// implements Prefix using [`Ipv4Addr`]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Ipv4Prefix {
    address: Ipv4Addr,
    length: u8,
}

impl std::fmt::Display for Ipv4Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.address)?;
        write!(f, "/")?;
        write!(f, "{}", self.length)
    }
}

impl std::str::FromStr for Ipv4Prefix {
    type Err = Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        let net = ipnet::Ipv4Net::from_str(s)?;
        Ok(unsafe { Self::unsafe_new(net.address(), net.length()) })
    }
}

impl From<ipnet::AddrParseError> for crate::errors::Error {
    fn from(err: ipnet::AddrParseError) -> Self {
        Self::ParseError(Some(Box::new(err)))
    }
}

impl Prefix for Ipv4Prefix {
    type Address = std::net::Ipv4Addr;

    fn address(&self) -> Self::Address {
        self.address
    }
    fn length(&self) -> u8 {
        self.length
    }
    unsafe fn unsafe_new(address: Self::Address, length: u8) -> Self {
        Ipv4Prefix { address, length }
    }
}

// https://stackoverflow.com/questions/53204327/how-to-have-a-private-part-of-a-trait
mod prefix_private {
    #[derive(Debug, PartialEq, Eq)]
    pub enum Child {
        Left,
        Right,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum PrefixOrd {
        Same,
        Contains,
        IsContained,
        Disjoint,
    }

    pub trait Cmp<P: super::Prefix> {
        fn containership(&self, longer: &P) -> (PrefixOrd, u8, Option<Child>);
        fn cmp(&self, b: &P) -> (PrefixOrd, bool, u8, Option<Child>);
    }
}

use prefix_private::{Child, Cmp, PrefixOrd};

impl<P, T> Cmp<T> for P
where
    P: Prefix,
    T: Prefix,
{
    // helper which compares to see if self contains the longer prefix.
    //
    // It assumes that self.length() <= longer.length(). Otherwise, the behavior is undefined.
    //
    // `ord`:    how self relates to the longer prefix (Same, Contains, Disjoint).
    //           since self is shorter than longer, IsContained is not possible.
    // `common`: number of leading bits that are equal in the two up to the shorter mask length.
    // `child`:  tells whether the first non-common bit in `longer` is a 0 (left) or 1 (right).
    //           It is only relevant if `exact` is false.
    //
    //  The following table describes how to interpret results:
    //
    // | ord      | common | child | note
    // |----------|--------|-------|-------
    // | Disjoint | 0..31  | Left  | the two are disjoint and `longer` compares less than `shorter`
    // | Disjoint | 0..31  | Right | the two are disjoint and `longer` compares greater than `shorter`
    // | Contains | 0..31  | Left  | `longer` should be `shorter`'s left child
    // | Contains | 0..31  | Right | `longer` should be `shorter`'s right child
    // | Same     | 0..32  | None  | `shorter` and `longer` are the same prefix
    fn containership(&self, longer: &T) -> (PrefixOrd, u8, Option<Child>) {
        let (short, long) = (self.address().octets(), longer.address().octets());

        for i in 0..4 {
            let offset = (i * 8) as u8;
            let short_len = self.length() - offset;
            let common = std::cmp::min(short_len, (short[i] ^ long[i]).leading_zeros() as u8);
            let ord = match short_len <= common {
                true => match short_len == longer.length() - offset {
                    true => break,
                    false => PrefixOrd::Contains,
                },
                false => match common == 8 {
                    true => continue,
                    false => PrefixOrd::Disjoint,
                },
            };
            let child = Some({
                let pivot_bit = match common == 8 {
                    true => 0x80 & long[i + 1],
                    false => 0x80 >> common & long[i],
                };
                match pivot_bit == 0 {
                    true => Child::Left,
                    false => Child::Right,
                }
            });
            return (ord, common + offset, child);
        }

        (PrefixOrd::Same, self.length(), None)
    }

    fn cmp(&self, other: &T) -> (PrefixOrd, bool, u8, Option<Child>) {
        let (reversed, (ord, common, child)) = match other.length() < self.length() {
            true => (true, other.containership(self)),
            false => (false, self.containership(other)),
        };
        let ord = match reversed && ord == PrefixOrd::Contains {
            true => PrefixOrd::IsContained,
            false => ord,
        };
        (ord, reversed, common, child)
    }
}

impl Prefix for ipnet::Ipv4Net {
    type Address = std::net::Ipv4Addr;

    fn address(&self) -> Self::Address {
        self.addr()
    }
    fn length(&self) -> u8 {
        self.prefix_len()
    }

    unsafe fn unsafe_new(ip: Self::Address, length: u8) -> Self {
        Self::new(ip, length).unwrap()
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
    /// # use addrs::ipv4::{Ipv4Prefix, Set};
    /// let prefix = "1.2.3.0/25".parse::<Ipv4Prefix>().unwrap();
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
    /// # use addrs::ipv4::{Ipv4Prefix, Set};
    /// let prefix = "1.2.3.0/25".parse::<Ipv4Prefix>().unwrap();
    /// assert_eq!(4, prefix.num_prefixes(27).unwrap());
    /// ```
    /// When the prefix length specifies a prefix having more than one address (e.g. a /24 in IPv4
    /// contains 256 addresses) then only properly aligned, wholly contained, prefixes of that size
    /// are counted. See the following example noting that there are many addresses on both ends of
    /// the range that cannot be included in a prefix of length 24 because of alignment.
    /// ```
    /// # use addrs::ipv4::{Ipv4Prefix, Set};
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
    /// # use addrs::ipv4::{Ipv4Prefix, Set};
    /// let prefix = "1.2.3.0/25".parse::<Ipv4Prefix>().unwrap();
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
    /// # use addrs::ipv4::{Ipv4Prefix, Prefix, Set};
    /// # use std::net::Ipv4Addr;
    /// // Contains self
    /// let net: Ipv4Prefix = "192.168.0.0/24".parse().unwrap();
    /// assert!(net.contains(&net));
    ///
    /// // Nets
    /// let net_yes: Ipv4Prefix = "192.168.0.0/25".parse().unwrap();
    /// let net_no: Ipv4Prefix = "192.168.0.0/23".parse().unwrap();
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

#[cfg(test)]
mod test {
    use super::prefix_private::{Child, Cmp, PrefixOrd};
    use super::*;

    fn cmp(
        a: util::Prefix,
        b: util::Prefix,
        expected_ord: PrefixOrd,
        expected_common: u8,
        expected_child: Option<Child>,
    ) {
        let (ord, common, child) = Cmp::containership(&a, &b);
        assert_eq!(expected_ord, ord);
        assert_eq!(expected_common, common);
        assert_eq!(expected_child, child);

        // compare forward
        let (ord, reversed, common, child) = Cmp::cmp(&a, &b);
        assert!(!reversed);
        assert_eq!(expected_common, common);
        assert_eq!(expected_child, child);
        assert_eq!(expected_ord, ord);

        // compare reversed
        let (ord, reversed, common_, _) = Cmp::cmp(&b, &a);
        assert_eq!(a.length() != b.length(), reversed);
        assert_eq!(expected_common, common_);
        let expected_ord = match expected_ord {
            PrefixOrd::Contains => PrefixOrd::IsContained,
            PrefixOrd::IsContained => PrefixOrd::Contains,
            _ => expected_ord,
        };
        assert_eq!(expected_ord, ord);
    }

    util::tests! { cmp {
        trivial(
            util::p("0.0.0.0/0"),
            util::p("0.0.0.0/0"),
            PrefixOrd::Same, 0, None);
        exact(
            util::p("10.0.0.0/16"),
            util::p("10.0.0.0/16"),
            PrefixOrd::Same, 16, None);
        exact_partial(
            util::p("10.0.0.0/19"),
            util::p("10.0.31.0/19"),
            PrefixOrd::Same, 19, None);
        empty_prefix_match(
            util::p("0.0.0.0/0"),
            util::p("10.10.0.0/16"),
            PrefixOrd::Contains, 0, Some(Child::Left));
        empty_prefix_match_backwards(
            util::p("0.0.0.0/0"),
            util::p("130.10.0.0/16"),
            PrefixOrd::Contains, 0, Some(Child::Right));
        matches(
            util::p("10.0.0.0/8"),
            util::p("10.10.0.0/16"),
            PrefixOrd::Contains, 8, Some(Child::Left));
        matches_partial(
            util::p("10.200.0.0/9"),
            util::p("10.129.0.0/16"),
            PrefixOrd::Contains, 9, Some(Child::Left));
        matches_backwards(
            util::p("10.0.0.0/8"),
            util::p("10.200.0.0/16"),
            PrefixOrd::Contains, 8, Some(Child::Right));
        matches_backwards_partial(
            util::p("10.240.0.0/9"),
            util::p("10.200.0.0/16"),
            PrefixOrd::Contains, 9, Some(Child::Right));
        disjoint(
            util::p("0.0.0.0/1"),
            util::p("128.0.0.0/1"),
            PrefixOrd::Disjoint, 0, Some(Child::Right));
        disjoint_longer(
            util::p("0.0.0.0/17"),
            util::p("0.0.128.0/17"),
            PrefixOrd::Disjoint, 16, Some(Child::Right));
        disjoint_longer_partial(
            util::p("0.0.0.0/17"),
            util::p("0.1.0.0/17"),
            PrefixOrd::Disjoint, 15, Some(Child::Right));
        disjoint_backwards(
            util::p("128.0.0.0/1"),
            util::p("0.0.0.0/1"),
            PrefixOrd::Disjoint, 0, Some(Child::Left));
        disjoint_backwards_longer(
            util::p("0.0.128.0/19"),
            util::p("0.0.0.0/19"),
            PrefixOrd::Disjoint, 16, Some(Child::Left));
        disjoint_backwards_longer_partial(
            util::p("0.1.0.0/19"),
            util::p("0.0.0.0/19"),
            PrefixOrd::Disjoint, 15, Some(Child::Left));
        disjoint_with_common(
            util::p("10.0.0.0/16"),
            util::p("10.10.0.0/16"),
            PrefixOrd::Disjoint, 12, Some(Child::Right));
        disjoint_with_more_disjoint_bytes(
            util::p("0.255.255.0/24"),
            util::p("128.0.0.0/24"),
            PrefixOrd::Disjoint, 0, Some(Child::Right));
    } }
}
